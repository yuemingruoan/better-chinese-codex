//! Shared popup-related constants for bottom pane widgets.

use crossterm::event::KeyCode;
use ratatui::text::Line;

use crate::i18n::tr;
use crate::key_hint;
use codex_protocol::config_types::Language;

/// Maximum number of rows any popup should attempt to display.
/// Keep this consistent across all popups for a uniform feel.
pub(crate) const MAX_POPUP_ROWS: usize = 8;

/// Standard footer hint text used by popups.
pub(crate) fn standard_popup_hint_line(language: Language) -> Line<'static> {
    Line::from(vec![
        tr(language, "popup.hint.press").into(),
        key_hint::plain(KeyCode::Enter).into(),
        tr(language, "popup.hint.confirm_or").into(),
        key_hint::plain(KeyCode::Esc).into(),
        tr(language, "popup.hint.go_back").into(),
    ])
}
