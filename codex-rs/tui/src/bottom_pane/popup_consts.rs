//! Shared popup-related constants for bottom pane widgets.

use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::key_hint;
use crate::key_hint::ShortcutHint;
use crate::keymap::ListAction;
use crate::keymap::ListKeymap;
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
        "select" => popup_hint_text("keymap-menu-hint-select", "select"),
        "back" => popup_hint_text("keymap-menu-hint-back", "back"),
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

/// Compact footer for shared pickers, using only the configured list actions.
pub(crate) fn picker_hint_line_for_keymap(list_keymap: &ListKeymap) -> Line<'static> {
    let mut spans = Vec::new();
    for (action, label) in [(ListAction::Accept, "select"), (ListAction::Cancel, "back")] {
        if let Some(hint) = list_keymap.primary_hint(action) {
            if !spans.is_empty() {
                spans.push(" · ".dim());
            }
            spans.extend(hint.spans());
            spans.push(format!(" {}", localized_hint_label(label)).dim());
        }
    }
    spans.into()
}

pub(crate) fn accept_cancel_hint_line(
    accept: Option<ShortcutHint>,
    accept_label: &'static str,
    cancel: Option<ShortcutHint>,
    cancel_label: &'static str,
) -> Line<'static> {
    let accept_label = localized_hint_label(accept_label);
    let cancel_label = localized_hint_label(cancel_label);
    let mut spans = Vec::new();
    if let Some(accept) = accept {
        spans.push(format!("{} ", popup_hint_text("popup-hint-press", "Press")).dim());
        spans.extend(accept.spans());
        spans.push(format!(" {accept_label}").dim());
    }
    if let Some(cancel) = cancel {
        spans.push(
            if spans.is_empty() {
                format!("{} ", popup_hint_text("popup-hint-press", "Press"))
            } else {
                format!(" {} ", popup_hint_text("popup-hint-or", "or"))
            }
            .dim(),
        );
        spans.extend(cancel.spans());
        spans.push(format!(" {cancel_label}").dim());
    }
    spans.into()
}
