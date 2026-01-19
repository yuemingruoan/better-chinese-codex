use codex_core::i18n;
use codex_protocol::config_types::Language;

pub(crate) fn tr(language: Language, key: &'static str) -> &'static str {
    i18n::tr(language, key)
}

pub(crate) fn tr_args(language: Language, key: &'static str, args: &[(&str, &str)]) -> String {
    i18n::tr_args(language, key, args)
}

pub(crate) fn tr_list(language: Language, key: &'static str) -> &'static [String] {
    i18n::tr_list(language, key)
}

pub(crate) fn language_name(ui_language: Language, target: Language) -> &'static str {
    i18n::language_name(ui_language, target)
}
