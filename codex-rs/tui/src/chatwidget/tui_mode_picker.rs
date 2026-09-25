//! Choose the next launch's transcript renderer without changing this session.

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionDescriptionLayout;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::picker_hint_line_for_keymap;

impl ChatWidget {
    pub(crate) fn show_tui_mode_picker(&mut self) {
        let items = [
            (
                false,
                crate::i18n::tr!("tui-mode-scrollback", "Scrollback"),
                crate::i18n::tr!(
                    "tui-mode-scrollback-description",
                    "Use your terminal's scrollback"
                ),
            ),
            (
                true,
                crate::i18n::tr!("tui-mode-fullscreen", "Fullscreen"),
                crate::i18n::tr!(
                    "tui-mode-fullscreen-description",
                    "Scroll within Codex's fullscreen view"
                ),
            ),
        ]
        .into_iter()
        .map(|(enabled, name, description)| SelectionItem {
            name: name.into(),
            description: Some(description.into()),
            is_current: enabled == self.local_settings.tui.fullscreen_transcript,
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::FullscreenTranscriptSelected { enabled });
            })],
            dismiss_on_select: true,
            require_explicit_confirmation: true,
            ..Default::default()
        })
        .collect();
        self.show_selection_view(SelectionViewParams {
            title: Some(crate::i18n::tr!("tui-mode-title", "TUI mode for next launch").into()),
            description_layout: SelectionDescriptionLayout::Columns,
            footer_note: Some(
                crate::i18n::tr!(
                    "tui-mode-restart",
                    "Restart to apply. Launch overrides still apply."
                )
                .into(),
            ),
            footer_hint: Some(picker_hint_line_for_keymap(&self.bottom_pane.list_keymap())),
            items,
            ..SelectionViewParams::picker()
        });
    }
}
