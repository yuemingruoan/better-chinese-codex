use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;
use glob::glob;
use serde::Deserialize;
use serde::Serialize;

use crate::function_tool::FunctionCallError;
use crate::i18n::tr;
use crate::i18n::tr_args;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::parse_arguments;
use crate::tools::handlers::read_file::IndentationArgs;
use crate::tools::handlers::read_file::read_indentation_block;
use crate::tools::handlers::read_file::read_slice;
use crate::tools::registry::ToolHandler;
use crate::tools::registry::ToolKind;
use codex_protocol::config_types::Language;
use codex_protocol::models::FunctionCallOutputBody;

const DEFAULT_LIMIT: usize = 2000;
const DEFAULT_OFFSET: usize = 1;
const MAX_FILES: usize = 20;
const MAX_TOTAL_LINES: usize = 50_000;

pub struct BatchesReadFileHandler;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct BatchesReadFileArgs {
    pub(crate) paths: Vec<PathSpec>,
    #[serde(default)]
    pub(crate) offset: Option<usize>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
    #[serde(default)]
    pub(crate) mode: Option<ReadMode>,
    #[serde(default)]
    pub(crate) indentation: Option<IndentationArgs>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct PathSpec {
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) offset: Option<usize>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
    #[serde(default)]
    pub(crate) mode: Option<ReadMode>,
    #[serde(default)]
    pub(crate) indentation: Option<IndentationArgs>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub(crate) enum ReadMode {
    #[default]
    Slice,
    Indentation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadOptions {
    pub(crate) offset: usize,
    pub(crate) limit: usize,
    pub(crate) mode: ReadMode,
    pub(crate) indentation: IndentationArgs,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            offset: DEFAULT_OFFSET,
            limit: DEFAULT_LIMIT,
            mode: ReadMode::default(),
            indentation: IndentationArgs::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedFile {
    pub(crate) path: PathBuf,
    pub(crate) options: ReadOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PathError {
    NotFound { path: String },
    NotFile { path: String },
    NoMatches { pattern: String },
    InvalidPattern { pattern: String },
    InvalidOffset { path: String },
    InvalidLimit { path: String },
    FileLimitExceeded { path: String },
    TotalLineLimitExceeded { path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PathResolution {
    File(ResolvedFile),
    Error(PathError),
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct BatchesReadFileEntry {
    path: String,
    success: bool,
    lines: Vec<String>,
    error: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct BatchesReadFileOutput {
    total_lines: usize,
    files: Vec<BatchesReadFileEntry>,
}

#[async_trait]
impl ToolHandler for BatchesReadFileHandler {
    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, FunctionCallError> {
        let ToolInvocation { payload, turn, .. } = invocation;
        let language = turn.config.language;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    tr(language, "batches_read_file.error.unsupported_payload").to_string(),
                ));
            }
        };

        let args: BatchesReadFileArgs = parse_arguments(&arguments)?;
        if args.paths.is_empty() {
            return Err(FunctionCallError::RespondToModel(
                tr(language, "batches_read_file.error.empty_paths").to_string(),
            ));
        }

        let offset = args.offset.unwrap_or(DEFAULT_OFFSET);
        if offset == 0 {
            return Err(FunctionCallError::RespondToModel(
                tr(language, "batches_read_file.error.invalid_offset").to_string(),
            ));
        }

        let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
        if limit == 0 {
            return Err(FunctionCallError::RespondToModel(
                tr(language, "batches_read_file.error.invalid_limit").to_string(),
            ));
        }

        let defaults = ReadOptions {
            offset,
            limit,
            mode: args.mode.unwrap_or_default(),
            indentation: args.indentation.unwrap_or_default(),
        };

        let resolved = expand_path_specs(&turn.cwd, &defaults, &args.paths);
        let output = read_resolutions(language, resolved, MAX_TOTAL_LINES).await;
        let success = output.files.iter().all(|entry| entry.success);
        let content = serde_json::to_string(&output).map_err(|err| {
            FunctionCallError::RespondToModel(tr_args(
                language,
                "batches_read_file.error.serialize_failed",
                &[("error", &err.to_string())],
            ))
        })?;

        Ok(ToolOutput::Function {
            body: FunctionCallOutputBody::Text(content),
            success: Some(success),
        })
    }
}

pub(crate) fn expand_path_specs(
    cwd: &Path,
    defaults: &ReadOptions,
    specs: &[PathSpec],
) -> Vec<PathResolution> {
    let mut resolved = Vec::new();
    for spec in specs {
        let merged_options = merge_options(defaults, spec);
        if merged_options.offset == 0 {
            resolved.push(PathResolution::Error(PathError::InvalidOffset {
                path: spec.path.clone(),
            }));
            continue;
        }
        if merged_options.limit == 0 {
            resolved.push(PathResolution::Error(PathError::InvalidLimit {
                path: spec.path.clone(),
            }));
            continue;
        }
        if contains_glob_chars(&spec.path) {
            let pattern = resolve_pattern(cwd, &spec.path);
            let matches = match glob(&pattern) {
                Ok(matches) => matches,
                Err(_) => {
                    resolved.push(PathResolution::Error(PathError::InvalidPattern {
                        pattern: spec.path.clone(),
                    }));
                    continue;
                }
            };

            let mut file_matches = Vec::new();
            for entry in matches.flatten() {
                if entry.is_file() {
                    file_matches.push(entry);
                }
            }
            file_matches.sort();
            if file_matches.is_empty() {
                resolved.push(PathResolution::Error(PathError::NoMatches {
                    pattern: spec.path.clone(),
                }));
            } else {
                resolved.extend(file_matches.into_iter().map(|path| {
                    PathResolution::File(ResolvedFile {
                        path,
                        options: merged_options.clone(),
                    })
                }));
            }
        } else {
            let resolved_path = resolve_path(cwd, &spec.path);
            if !resolved_path.exists() {
                resolved.push(PathResolution::Error(PathError::NotFound {
                    path: spec.path.clone(),
                }));
                continue;
            }
            if !resolved_path.is_file() {
                resolved.push(PathResolution::Error(PathError::NotFile {
                    path: spec.path.clone(),
                }));
                continue;
            }
            resolved.push(PathResolution::File(ResolvedFile {
                path: resolved_path,
                options: merged_options,
            }));
        }
    }

    apply_file_limit(&mut resolved);
    resolved
}

async fn read_resolutions(
    language: Language,
    resolutions: Vec<PathResolution>,
    total_line_limit: usize,
) -> BatchesReadFileOutput {
    let mut total_lines = 0usize;
    let mut entries = Vec::with_capacity(resolutions.len());

    for resolution in resolutions {
        match resolution {
            PathResolution::Error(error) => {
                entries.push(error_entry(
                    error_path(&error),
                    path_error_message(language, &error),
                ));
            }
            PathResolution::File(resolved_file) => {
                let remaining = total_line_limit.saturating_sub(total_lines);
                if remaining == 0 {
                    let error = PathError::TotalLineLimitExceeded {
                        path: resolved_file.path.display().to_string(),
                    };
                    entries.push(error_entry(
                        error_path(&error),
                        path_error_message(language, &error),
                    ));
                    continue;
                }
                let effective_limit = resolved_file.options.limit.min(remaining);
                let result = match resolved_file.options.mode {
                    ReadMode::Slice => {
                        read_slice(
                            &resolved_file.path,
                            resolved_file.options.offset,
                            effective_limit,
                        )
                        .await
                    }
                    ReadMode::Indentation => {
                        read_indentation_block(
                            &resolved_file.path,
                            resolved_file.options.offset,
                            effective_limit,
                            resolved_file.options.indentation.clone(),
                        )
                        .await
                    }
                };

                match result {
                    Ok(lines) => {
                        let new_total = total_lines + lines.len();
                        let mut error = None;
                        if effective_limit < resolved_file.options.limit
                            && lines.len() == effective_limit
                            && new_total >= total_line_limit
                        {
                            error = Some(path_error_message(
                                language,
                                &PathError::TotalLineLimitExceeded {
                                    path: resolved_file.path.display().to_string(),
                                },
                            ));
                        }
                        total_lines = new_total;
                        entries.push(BatchesReadFileEntry {
                            path: resolved_file.path.display().to_string(),
                            success: error.is_none(),
                            lines,
                            error,
                        });
                    }
                    Err(err) => {
                        entries.push(error_entry(
                            resolved_file.path.display().to_string(),
                            err.to_string(),
                        ));
                    }
                }
            }
        }
    }

    BatchesReadFileOutput {
        total_lines,
        files: entries,
    }
}

fn merge_options(defaults: &ReadOptions, spec: &PathSpec) -> ReadOptions {
    let mut merged = defaults.clone();
    if let Some(offset) = spec.offset {
        merged.offset = offset;
    }
    if let Some(limit) = spec.limit {
        merged.limit = limit;
    }
    if let Some(mode) = spec.mode {
        merged.mode = mode;
    }
    if let Some(indentation) = spec.indentation.clone() {
        merged.indentation = indentation;
    }
    merged
}

fn error_entry(path: String, message: String) -> BatchesReadFileEntry {
    BatchesReadFileEntry {
        path,
        success: false,
        lines: Vec::new(),
        error: Some(message),
    }
}

fn error_path(error: &PathError) -> String {
    match error {
        PathError::NotFound { path }
        | PathError::NotFile { path }
        | PathError::InvalidOffset { path }
        | PathError::InvalidLimit { path }
        | PathError::FileLimitExceeded { path }
        | PathError::TotalLineLimitExceeded { path } => path.clone(),
        PathError::NoMatches { pattern } | PathError::InvalidPattern { pattern } => pattern.clone(),
    }
}

fn path_error_message(language: Language, error: &PathError) -> String {
    match error {
        PathError::NotFound { path } => tr_args(
            language,
            "batches_read_file.error.file_not_found",
            &[("path", path)],
        ),
        PathError::NotFile { path } => tr_args(
            language,
            "batches_read_file.error.not_file",
            &[("path", path)],
        ),
        PathError::NoMatches { pattern } => tr_args(
            language,
            "batches_read_file.error.no_matches",
            &[("pattern", pattern)],
        ),
        PathError::InvalidPattern { pattern } => tr_args(
            language,
            "batches_read_file.error.invalid_pattern",
            &[("pattern", pattern)],
        ),
        PathError::InvalidOffset { path } => tr_args(
            language,
            "batches_read_file.error.file_invalid_offset",
            &[("path", path)],
        ),
        PathError::InvalidLimit { path } => tr_args(
            language,
            "batches_read_file.error.file_invalid_limit",
            &[("path", path)],
        ),
        PathError::FileLimitExceeded { path } => tr_args(
            language,
            "batches_read_file.error.file_limit_exceeded",
            &[("path", path)],
        ),
        PathError::TotalLineLimitExceeded { path } => tr_args(
            language,
            "batches_read_file.error.total_line_limit",
            &[("path", path)],
        ),
    }
}

fn apply_file_limit(entries: &mut [PathResolution]) {
    let mut file_count = 0usize;
    for entry in entries {
        let resolved_file = match entry {
            PathResolution::File(resolved_file) => resolved_file,
            PathResolution::Error(_) => continue,
        };
        if file_count < MAX_FILES {
            file_count += 1;
            continue;
        }
        *entry = PathResolution::Error(PathError::FileLimitExceeded {
            path: resolved_file.path.display().to_string(),
        });
    }
}

fn resolve_path(cwd: &Path, raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn resolve_pattern(cwd: &Path, pattern: &str) -> String {
    if Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        cwd.join(pattern).to_string_lossy().to_string()
    }
}

fn contains_glob_chars(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::config_types::Language;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn expands_relative_glob_sorted() {
        let temp = tempdir().expect("create temp dir");
        let dir = temp.path();
        std::fs::write(dir.join("b.txt"), "b").expect("write b");
        std::fs::write(dir.join("a.txt"), "a").expect("write a");
        let specs = vec![PathSpec {
            path: "*.txt".to_string(),
            offset: None,
            limit: None,
            mode: None,
            indentation: None,
        }];

        let results = expand_path_specs(dir, &ReadOptions::default(), &specs);
        let expected = vec![
            PathResolution::File(ResolvedFile {
                path: dir.join("a.txt"),
                options: ReadOptions::default(),
            }),
            PathResolution::File(ResolvedFile {
                path: dir.join("b.txt"),
                options: ReadOptions::default(),
            }),
        ];
        assert_eq!(results, expected);
    }

    #[test]
    fn returns_no_matches_error() {
        let temp = tempdir().expect("create temp dir");
        let dir = temp.path();
        let specs = vec![PathSpec {
            path: "missing-*.txt".to_string(),
            offset: None,
            limit: None,
            mode: None,
            indentation: None,
        }];

        let results = expand_path_specs(dir, &ReadOptions::default(), &specs);
        assert_eq!(
            results,
            vec![PathResolution::Error(PathError::NoMatches {
                pattern: "missing-*.txt".to_string()
            })]
        );
    }

    #[test]
    fn file_limit_converts_extra_entries_to_errors() {
        let temp = tempdir().expect("create temp dir");
        let dir = temp.path();
        for index in 0..=20 {
            std::fs::write(dir.join(format!("file{index:02}.txt")), "x").expect("write file");
        }
        let specs = vec![PathSpec {
            path: "*.txt".to_string(),
            offset: None,
            limit: None,
            mode: None,
            indentation: None,
        }];

        let results = expand_path_specs(dir, &ReadOptions::default(), &specs);
        assert_eq!(results.len(), 21);
        let (allowed, remainder) = results.split_at(20);
        assert!(
            allowed
                .iter()
                .all(|entry| matches!(entry, PathResolution::File(_)))
        );
        assert_eq!(remainder.len(), 1);
        assert!(matches!(
            remainder[0],
            PathResolution::Error(PathError::FileLimitExceeded { .. })
        ));
    }

    #[tokio::test]
    async fn reads_files_and_tracks_total_lines() {
        let temp = tempdir().expect("create temp dir");
        let dir = temp.path();
        let first = dir.join("one.txt");
        let second = dir.join("two.txt");
        std::fs::write(&first, "alpha\nbeta\n").expect("write one");
        std::fs::write(&second, "gamma\ndelta\n").expect("write two");
        let specs = vec![
            PathSpec {
                path: "one.txt".to_string(),
                offset: None,
                limit: None,
                mode: None,
                indentation: None,
            },
            PathSpec {
                path: "two.txt".to_string(),
                offset: None,
                limit: None,
                mode: None,
                indentation: None,
            },
        ];

        let resolutions = expand_path_specs(dir, &ReadOptions::default(), &specs);
        let output = read_resolutions(Language::En, resolutions, 10).await;
        let expected = BatchesReadFileOutput {
            total_lines: 4,
            files: vec![
                BatchesReadFileEntry {
                    path: first.display().to_string(),
                    success: true,
                    lines: vec!["L1: alpha".to_string(), "L2: beta".to_string()],
                    error: None,
                },
                BatchesReadFileEntry {
                    path: second.display().to_string(),
                    success: true,
                    lines: vec!["L1: gamma".to_string(), "L2: delta".to_string()],
                    error: None,
                },
            ],
        };
        assert_eq!(output, expected);
    }

    #[tokio::test]
    async fn truncates_when_total_line_limit_reached() {
        let temp = tempdir().expect("create temp dir");
        let dir = temp.path();
        let first = dir.join("one.txt");
        let second = dir.join("two.txt");
        std::fs::write(&first, "alpha\nbeta\n").expect("write one");
        std::fs::write(&second, "gamma\ndelta\n").expect("write two");
        let specs = vec![
            PathSpec {
                path: "one.txt".to_string(),
                offset: None,
                limit: None,
                mode: None,
                indentation: None,
            },
            PathSpec {
                path: "two.txt".to_string(),
                offset: None,
                limit: None,
                mode: None,
                indentation: None,
            },
        ];

        let resolutions = expand_path_specs(dir, &ReadOptions::default(), &specs);
        let output = read_resolutions(Language::En, resolutions, 3).await;
        assert_eq!(output.total_lines, 3);
        assert_eq!(output.files.len(), 2);
        assert_eq!(output.files[0].lines.len(), 2);
        assert!(output.files[0].success);
        assert!(output.files[0].error.is_none());
        assert_eq!(output.files[1].lines.len(), 1);
        assert!(!output.files[1].success);
        assert!(output.files[1].error.is_some());
        assert_eq!(output.files[0].path, first.display().to_string());
        assert_eq!(output.files[1].path, second.display().to_string());
    }
}
