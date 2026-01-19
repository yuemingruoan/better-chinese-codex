use codex_protocol::config_types::Language;
use once_cell::sync::Lazy;
#[cfg(test)]
use std::collections::BTreeSet;
use std::collections::HashMap;
use toml::Value;

static EN_CATALOG: Lazy<Catalog> = Lazy::new(|| Catalog::from_str(include_str!("../i18n/en.toml")));
static ZH_CATALOG: Lazy<Catalog> =
    Lazy::new(|| Catalog::from_str(include_str!("../i18n/zh-cn.toml")));

pub fn tr(language: Language, key: &'static str) -> &'static str {
    lookup(language, key).unwrap_or(key)
}

pub fn tr_args(language: Language, key: &'static str, args: &[(&str, &str)]) -> String {
    let template = tr(language, key);
    interpolate(template, args)
}

pub fn tr_list(language: Language, key: &'static str) -> &'static [String] {
    if let Some(list) = lookup_list(language, key) {
        return list;
    }
    if let Some(list) = lookup_list(Language::En, key) {
        return list;
    }
    static EMPTY: Lazy<Vec<String>> = Lazy::new(Vec::new);
    EMPTY.as_slice()
}

pub fn language_name(ui_language: Language, target: Language) -> &'static str {
    match target {
        Language::En => tr(ui_language, "language.name.en"),
        Language::ZhCn => tr(ui_language, "language.name.zh_cn"),
    }
}

fn lookup(language: Language, key: &'static str) -> Option<&'static str> {
    catalog(language)
        .strings
        .get(key)
        .map(String::as_str)
        .or_else(|| catalog(Language::En).strings.get(key).map(String::as_str))
}

fn lookup_list(language: Language, key: &'static str) -> Option<&'static [String]> {
    catalog(language).lists.get(key).map(Vec::as_slice)
}

fn catalog(language: Language) -> &'static Catalog {
    match language {
        Language::En => &EN_CATALOG,
        Language::ZhCn => &ZH_CATALOG,
    }
}

#[derive(Debug)]
struct Catalog {
    strings: HashMap<String, String>,
    lists: HashMap<String, Vec<String>>,
}

impl Catalog {
    fn from_str(raw: &str) -> Self {
        let value: Value =
            toml::from_str(raw).unwrap_or_else(|err| panic!("i18n TOML must parse: {err}"));
        let mut catalog = Catalog {
            strings: HashMap::new(),
            lists: HashMap::new(),
        };
        flatten(None, &value, &mut catalog);
        catalog
    }

    #[cfg(test)]
    fn keys(&self) -> BTreeSet<String> {
        self.strings
            .keys()
            .chain(self.lists.keys())
            .cloned()
            .collect()
    }
}

fn flatten(prefix: Option<&str>, value: &Value, catalog: &mut Catalog) {
    match value {
        Value::Table(table) => {
            for (key, entry) in table {
                let next = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                flatten(Some(&next), entry, catalog);
            }
        }
        Value::String(text) => {
            if let Some(prefix) = prefix {
                insert_string(prefix, text, catalog);
            } else {
                panic!("i18n values must be keyed");
            }
        }
        Value::Array(values) => {
            if let Some(prefix) = prefix {
                let mut items = Vec::with_capacity(values.len());
                for entry in values {
                    match entry {
                        Value::String(text) => items.push(text.clone()),
                        _ => panic!("i18n list values must be strings"),
                    }
                }
                insert_list(prefix, items, catalog);
            } else {
                panic!("i18n lists must be keyed");
            }
        }
        _ => panic!("unsupported i18n value type"),
    }
}

fn insert_string(key: &str, value: &str, catalog: &mut Catalog) {
    let previous = catalog.strings.insert(key.to_string(), value.to_string());
    if previous.is_some() {
        panic!("duplicate i18n key: {key}");
    }
}

fn insert_list(key: &str, value: Vec<String>, catalog: &mut Catalog) {
    let previous = catalog.lists.insert(key.to_string(), value);
    if previous.is_some() {
        panic!("duplicate i18n list key: {key}");
    }
}

fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("${") {
        let (before, after_start) = rest.split_at(start);
        out.push_str(before);
        let after_start = &after_start[2..];
        if let Some(end) = after_start.find('}') {
            let key = &after_start[..end];
            if let Some((_, value)) = args.iter().find(|(name, _)| *name == key) {
                out.push_str(value);
            } else {
                out.push_str("${");
                out.push_str(key);
                out.push('}');
            }
            rest = &after_start[end + 1..];
        } else {
            out.push_str("${");
            out.push_str(after_start);
            return out;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_share_keys() {
        let en_keys = EN_CATALOG.keys();
        let zh_keys = ZH_CATALOG.keys();
        assert_eq!(en_keys, zh_keys);
    }

    #[test]
    fn interpolate_replaces_placeholders() {
        let rendered = interpolate("Hello ${name}", &[("name", "World")]);
        assert_eq!(rendered, "Hello World");
    }
}
