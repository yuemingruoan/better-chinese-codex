use codex_protocol::config_types::Language;

pub(crate) fn tr(language: Language, zh: &'static str, en: &'static str) -> &'static str {
    match language {
        Language::ZhCn => zh,
        Language::En => en,
    }
}

pub(crate) fn language_name(ui_language: Language, target: Language) -> &'static str {
    match (ui_language, target) {
        (Language::ZhCn, Language::En) => "英文",
        (Language::En, Language::En) => "English",
        (Language::ZhCn, Language::ZhCn) => "简体中文",
        (Language::En, Language::ZhCn) => "Simplified Chinese",
    }
}
