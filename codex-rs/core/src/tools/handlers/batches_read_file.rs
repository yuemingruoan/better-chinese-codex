use std::path::Path;
use std::path::PathBuf;

use glob::glob;
use serde::Deserialize;

const DEFAULT_LIMIT: usize = 2000;
const DEFAULT_OFFSET: usize = 1;
const MAX_FILES: usize = 20;

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
pub(crate) enum ReadMode {
    Slice,
    Indentation,
}

impl Default for ReadMode {
    fn default() -> Self {
        Self::Slice
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct IndentationArgs {
    #[serde(default)]
    pub(crate) anchor_line: Option<usize>,
    #[serde(default = "defaults::max_levels")]
    pub(crate) max_levels: usize,
    #[serde(default = "defaults::include_siblings")]
    pub(crate) include_siblings: bool,
    #[serde(default = "defaults::include_header")]
    pub(crate) include_header: bool,
    #[serde(default)]
    pub(crate) max_lines: Option<usize>,
}

impl Default for IndentationArgs {
    fn default() -> Self {
        Self {
            anchor_line: None,
            max_levels: defaults::max_levels(),
            include_siblings: defaults::include_siblings(),
            include_header: defaults::include_header(),
            max_lines: None,
        }
    }
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
    FileLimitExceeded { path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PathResolution {
    File(ResolvedFile),
    Error(PathError),
}

pub(crate) fn expand_path_specs(
    cwd: &Path,
    defaults: &ReadOptions,
    specs: &[PathSpec],
) -> Vec<PathResolution> {
    let mut resolved = Vec::new();
    for spec in specs {
        let merged_options = merge_options(defaults, spec);
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

mod defaults {
    pub fn max_levels() -> usize {
        0
    }

    pub fn include_siblings() -> bool {
        false
    }

    pub fn include_header() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
