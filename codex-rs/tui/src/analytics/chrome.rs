//! Report navigation and bounded keyboard/mouse help for the full-screen usage view.
//! Each report retains its own reading position; help never replaces its selection or draft.

use super::AnalyticsView;
use super::controls::Control;
use super::sections::Section;
use crate::keymap::ListAction;
use ratatui::text::Line;

impl AnalyticsView {
    pub(super) fn tab_label(&self, section: Section) -> &'static str {
        match section {
            Section::Summary => crate::i18n::tr!("analytics-overview", "Overview"),
            Section::Usage if self.business() => crate::i18n::tr!("analytics-tokens", "Tokens"),
            Section::Usage => crate::i18n::tr!("analytics-usage", "Usage"),
            Section::Credits => crate::i18n::tr!("analytics-credits", "Credits"),
            Section::Activity => crate::i18n::tr!("analytics-messages", "Messages"),
            Section::Plugins => crate::i18n::tr!("analytics-plugins", "Plugins"),
            Section::Skills => crate::i18n::tr!("analytics-skills", "Skills"),
            Section::Chats => crate::i18n::tr!("analytics-chats", "Chats"),
            Section::Plan => crate::i18n::tr!("analytics-plan", "Plan"),
        }
    }

    /// Each surface owns its reading position; switching surfaces never copies it.
    pub(super) fn scroll_offset(&self) -> usize {
        if self.show_help {
            self.help_scroll
        } else if self.zoomed {
            self.sections[self.section].scroll_offset
        } else {
            self.dashboard_scroll
        }
    }

    pub(super) fn scroll_offset_mut(&mut self) -> &mut usize {
        if self.show_help {
            &mut self.help_scroll
        } else if self.zoomed {
            &mut self.sections[self.section].scroll_offset
        } else {
            &mut self.dashboard_scroll
        }
    }

    pub(super) fn toggle_dashboard(&mut self) {
        self.zoomed = !self.zoomed;
        self.follow_selection = !self.zoomed;
    }

    pub(super) fn select_section(&mut self, section: Section) {
        if section != self.section {
            self.section = section;
            self.follow_selection = !self.zoomed;
            self.show_help = false;
        }
    }

    pub(super) fn chat_has_details(&self) -> bool {
        let cursor = self.sections[Section::Chats].cursor;
        if self.business() {
            self.chats
                .ready()
                .and_then(|chats| chats.rows.get(cursor))
                .and_then(|chat| chat.usage.as_ref())
                .is_some()
        } else {
            self.task_rows()
                .get(cursor)
                .and_then(|chat| super::task_panel::available(chat))
                .is_some()
        }
    }

    pub(super) fn hint(&self, action: ListAction) -> String {
        self.keymap
            .primary_hint(action)
            .map(crate::key_hint::ShortcutHint::display_label)
            .unwrap_or_default()
    }

    pub(super) fn help_shortcut_available(&self) -> bool {
        [
            crossterm::event::KeyModifiers::NONE,
            crossterm::event::KeyModifiers::SHIFT,
        ]
        .into_iter()
        .all(|modifiers| {
            self.keymap
                .action_for(crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char('?'),
                    modifiers,
                ))
                .is_none()
        })
    }

    pub(super) fn help_lines(&self, width: usize) -> Vec<Line<'static>> {
        let mut controls = vec![
            crate::i18n::tr!(
                "analytics-help-reports",
                "Tab / Shift+Tab · next / previous report"
            )
            .to_owned(),
            crate::i18n::tr_format!(
                "analytics-help-report-number",
                "1–{value} · open a report directly",
                value = self.visible_sections().len()
            ),
            crate::i18n::tr_format!(
                "analytics-help-chart-selection",
                "{value} / {value2} · select chart day or plan window",
                value = self.hint(ListAction::MoveLeft),
                value2 = self.hint(ListAction::MoveRight)
            ),
            crate::i18n::tr_format!(
                "analytics-help-period-selection",
                "{value} / {value2} · select chat or plan period; scroll Overview",
                value = self.hint(ListAction::MoveUp),
                value2 = self.hint(ListAction::MoveDown)
            ),
            crate::i18n::tr_format!(
                "analytics-help-expand",
                "{value} · expand or collapse details",
                value = self.hint(ListAction::Accept)
            ),
            crate::i18n::tr_format!(
                "analytics-help-scroll",
                "{value} / {value2} · scroll report",
                value = self.hint(ListAction::PageUp),
                value2 = self.hint(ListAction::PageDown)
            ),
            crate::i18n::tr!(
                "analytics-help-mouse",
                "In alternate screen: mouse wheel · scroll; click tabs or report controls"
            )
            .into(),
        ];
        if self.control_available(Control::Range) {
            controls.push(
                crate::i18n::tr!("analytics-help-range", "r · switch between 7 and 30 days").into(),
            );
        }
        if self.control_available(Control::Group) {
            controls.push(
                crate::i18n::tr!(
                    "analytics-help-group",
                    "g · change grouping / Overview aggregation"
                )
                .into(),
            );
        }
        if self.control_available(Control::Model) {
            controls
                .push(crate::i18n::tr!("analytics-help-model", "m · cycle model filter").into());
        }
        if self.control_available(Control::TaskMetric) {
            controls.push(
                crate::i18n::tr!("analytics-help-sort", "s · change chat sort metric").into(),
            );
        }
        if self.control_available(Control::ZeroCreditGroups) {
            controls.push(
                crate::i18n::tr!(
                    "analytics-help-zeros",
                    "a · show zero-credit groups in expanded details"
                )
                .into(),
            );
        }
        controls.extend([
            crate::i18n::tr!("analytics-help-refresh", "R · refresh all reports").into(),
            crate::i18n::tr_format!(
                "analytics-help-focus",
                "z · dashboard / focused report; {value} · focus dashboard card",
                value = self.hint(ListAction::Accept)
            ),
            crate::i18n::tr_format!(
                "analytics-help-back",
                "{value} · back; q / ctrl+c · close usage",
                value = self.hint(ListAction::Cancel)
            ),
        ]);
        if self.help_shortcut_available() {
            controls.insert(
                controls.len() - 1,
                crate::i18n::tr!("analytics-help-toggle", "? · toggle this help").into(),
            );
        }
        if let Some(updated) = self.sections[self.section]
            .history
            .ready()
            .and_then(|report| report.updated_at)
            .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, /*nsecs*/ 0))
        {
            controls.push(crate::i18n::tr_format!(
                "analytics-report-updated",
                "Report updated {value} UTC",
                value = updated.format(self.clock_format.date_time_format())
            ));
        }
        controls
            .into_iter()
            .flat_map(|line| {
                textwrap::wrap(&line, width)
                    .into_iter()
                    .map(|line| Line::from(line.into_owned()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
