//! Claude-style welcome layout and composer rules, selected by the external CCU theme.
//! The card stays in scrollback; narrow terminals collapse its two columns into a compact card.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Widget;

use crate::ccu_theme::CcuTheme;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use crate::wrapping::RtOptions;
use crate::wrapping::word_wrap_line;

pub(crate) struct WelcomeCard<'a> {
    pub version: &'a str,
    pub model: &'a str,
    pub reasoning: Option<&'a str>,
    pub directory: &'a str,
    pub yolo_mode: bool,
    pub fast: bool,
}

impl WelcomeCard<'_> {
    pub(crate) fn lines(&self, width: u16, theme: &CcuTheme) -> Vec<Line<'static>> {
        if width < 6 {
            return vec![truncate_line_with_ellipsis_if_overflow(
                "Codex".into(),
                usize::from(width),
            )];
        }
        let inner = usize::from(width) - 4;
        let border = theme.welcome_style("border").unwrap_or_default();
        let accent = theme.welcome_style("title").unwrap_or_default();
        let foreground = theme.interface_style("foreground").unwrap_or_default();
        let muted = theme.interface_style("muted").unwrap_or_default();
        let text =
            |key, fallback: &str| crate::i18n::global().text(key, None, || fallback.to_string());
        let model = format!(
            "{}{}{}",
            self.model,
            self.reasoning
                .map(|effort| format!(" · {effort}"))
                .unwrap_or_default(),
            if self.fast { " · fast" } else { "" }
        );
        let permissions = if self.yolo_mode {
            text("ccu-welcome-full-access", "Full access")
        } else {
            text("ccu-welcome-permissions", "/permissions to review access")
        };
        let mut title = vec![
            Span::styled(" Codex CCU ", accent.bold()),
            Span::styled(format!("v{} ", self.version), muted),
        ];
        let title =
            truncate_line_with_ellipsis_if_overflow(Line::from(std::mem::take(&mut title)), inner);
        let mut top = vec![Span::styled("╭─", border)];
        top.extend(title.spans.clone());
        top.push(Span::styled(
            format!("{}╮", "─".repeat(inner + 1 - title.width())),
            border,
        ));
        let mut lines = vec![Line::from(top)];

        if width >= 100 {
            let left_width = (inner / 4).max(26);
            let right_width = inner - left_width - 3;
            let left = vec![
                Line::default(),
                Line::styled(text("ccu-welcome-back", "Welcome back!"), foreground.bold()),
                Line::default(),
                Line::styled(" ▄▄▄▄▄ ", accent),
                Line::styled(" █ >_█ ", accent),
                Line::styled(" ▀▀ ▀▀ ", accent),
                Line::default(),
                Line::styled(model, foreground),
                Line::styled(self.directory.to_string(), muted),
                Line::styled(
                    permissions,
                    theme.welcome_style("permissions").unwrap_or(accent),
                ),
            ];
            let mut right = vec![Line::styled(
                text("ccu-welcome-tips", "Tips for getting started"),
                accent.bold(),
            )];
            let init = Line::from(vec![
                Span::styled("/init  ", foreground),
                Span::styled(
                    text(
                        "session-help-init",
                        "create an AGENTS.md file with instructions for Codex",
                    ),
                    foreground,
                ),
            ]);
            right.extend(
                word_wrap_line(&init, RtOptions::new(right_width))
                    .iter()
                    .map(crate::render::line_utils::line_to_static),
            );
            right.push(Line::styled("─".repeat(right_width), border));
            right.push(Line::styled(
                text("ccu-welcome-commands", "Useful commands"),
                accent.bold(),
            ));
            for (command, key, fallback) in [
                (
                    "/model",
                    "session-help-model",
                    "choose a model and reasoning effort",
                ),
                (
                    "/permissions",
                    "session-help-permissions",
                    "choose what Codex is allowed to do",
                ),
                (
                    "/status",
                    "session-help-status",
                    "show current session configuration",
                ),
            ] {
                let line = Line::from(vec![
                    Span::styled(format!("{command}  "), foreground),
                    Span::styled(text(key, fallback), muted),
                ]);
                right.extend(
                    word_wrap_line(&line, RtOptions::new(right_width))
                        .iter()
                        .map(crate::render::line_utils::line_to_static),
                );
            }
            for index in 0..left.len().max(right.len()) {
                let left = padded(
                    left.get(index).cloned().unwrap_or_default(),
                    left_width,
                    true,
                );
                let right = padded(
                    right.get(index).cloned().unwrap_or_default(),
                    right_width,
                    false,
                );
                let mut row = vec![Span::styled("│ ", border)];
                row.extend(left.spans);
                row.push(Span::styled(" │ ", border));
                row.extend(right.spans);
                row.push(Span::styled(" │", border));
                lines.push(Line::from(row));
            }
        } else {
            for line in [
                Line::styled(text("ccu-welcome-back", "Welcome back!"), foreground.bold()),
                Line::styled(format!(">_  {model}"), foreground),
                Line::styled(self.directory.to_string(), muted),
                Line::styled(permissions, accent),
                Line::styled("/init · /model · /permissions · /status", muted),
            ] {
                let mut row = vec![Span::styled("│ ", border)];
                row.extend(padded(line, inner, false).spans);
                row.push(Span::styled(" │", border));
                lines.push(Line::from(row));
            }
        }
        lines.push(Line::styled(format!("╰{}╯", "─".repeat(inner + 2)), border));
        lines
    }
}

fn padded(line: Line<'static>, width: usize, centered: bool) -> Line<'static> {
    let line = truncate_line_with_ellipsis_if_overflow(line, width);
    let remaining = width.saturating_sub(line.width());
    let before = if centered { remaining / 2 } else { 0 };
    let mut spans = vec![Span::raw(" ".repeat(before))];
    let style = line.style;
    spans.extend(line.spans.into_iter().map(|span| {
        let combined = style.patch(span.style);
        span.style(combined)
    }));
    spans.push(Span::raw(" ".repeat(remaining - before)));
    Line::from(spans)
}

pub(crate) fn render_composer_rules(area: Rect, buf: &mut Buffer, theme: &CcuTheme) {
    if area.height < 3 {
        return;
    }
    let rule = Line::styled(
        "─".repeat(usize::from(area.width)),
        theme.interface_style("rule").unwrap_or_default(),
    );
    for y in [area.y, area.bottom() - 1] {
        rule.clone()
            .render(Rect::new(area.x, y, area.width, 1), buf);
    }
}
