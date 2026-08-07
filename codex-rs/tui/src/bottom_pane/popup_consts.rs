//! Shared popup-related constants for bottom pane widgets.

use ratatui::text::Line;

use crate::key_hint;
use crate::key_hint::KeyBinding;
use crate::keymap::ListKeymap;
use crate::keymap::primary_binding;
use crossterm::event::KeyCode;

/// Maximum number of rows any popup should attempt to display.
/// Keep this consistent across all popups for a uniform feel.
pub(crate) const MAX_POPUP_ROWS: usize = 8;

fn popup_hint_text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

fn localized_hint_label(label: &'static str) -> String {
    match label {
        "to confirm" => popup_hint_text("popup-hint-confirm", "to confirm"),
        "to go back" => popup_hint_text("popup-hint-go-back", "to go back"),
        "to cancel" => popup_hint_text("popup-hint-cancel", "to cancel"),
        _ => label.to_string(),
    }
}

/// Standard footer hint text used by popups.
pub(crate) fn standard_popup_hint_line() -> Line<'static> {
    let press = popup_hint_text("popup-hint-press", "Press");
    let confirm = popup_hint_text("popup-hint-confirm", "to confirm");
    let or = popup_hint_text("popup-hint-or", "or");
    let go_back = popup_hint_text("popup-hint-go-back", "to go back");
    Line::from(vec![
        format!("{press} ").into(),
        key_hint::plain(KeyCode::Enter).into(),
        format!(" {confirm} {or} ").into(),
        key_hint::plain(KeyCode::Esc).into(),
        format!(" {go_back}").into(),
    ])
}

pub(crate) fn standard_popup_hint_line_for_keymap(list_keymap: &ListKeymap) -> Line<'static> {
    accept_cancel_hint_line(
        primary_binding(&list_keymap.accept),
        "to confirm",
        primary_binding(&list_keymap.cancel),
        "to go back",
    )
}

pub(crate) fn accept_cancel_hint_line(
    accept: Option<KeyBinding>,
    accept_label: &'static str,
    cancel: Option<KeyBinding>,
    cancel_label: &'static str,
) -> Line<'static> {
    let press = popup_hint_text("popup-hint-press", "Press");
    let or = popup_hint_text("popup-hint-or", "or");
    let accept_label = localized_hint_label(accept_label);
    let cancel_label = localized_hint_label(cancel_label);
    match (accept, cancel) {
        (Some(accept), Some(cancel)) => Line::from(vec![
            format!("{press} ").into(),
            accept.into(),
            format!(" {accept_label} {or} ").into(),
            cancel.into(),
            format!(" {cancel_label}").into(),
        ]),
        (Some(accept), None) => Line::from(vec![
            format!("{press} ").into(),
            accept.into(),
            format!(" {accept_label}").into(),
        ]),
        (None, Some(cancel)) => Line::from(vec![
            format!("{press} ").into(),
            cancel.into(),
            format!(" {cancel_label}").into(),
        ]),
        (None, None) => Line::from(""),
    }
}
