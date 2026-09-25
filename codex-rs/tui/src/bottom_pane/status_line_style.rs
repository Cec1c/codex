//! Theme-derived styling for the configurable footer statusline.

use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Styled;
use ratatui::text::Line;
use ratatui::text::Span;

use super::status_line_setup::StatusLineItem;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use crate::render::highlight::foreground_style_for_scopes;
use crate::style::readable_color_on;
use crate::style::secondary_text_style;
use crate::thread_color::thread_color;
use codex_protocol::ThreadId;

const STATUS_LINE_SEPARATOR: &str = " · ";
const STATUS_LINE_COLOR_SATURATION_PERCENT: u16 = 85;
const STATUS_LINE_COLOR_BRIGHTNESS_PERCENT: u16 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StatusLineAccent {
    Model,
    Path,
    Branch,
    State,
    Usage,
    Limit,
    Metadata,
    Mode,
    Thread,
    Progress,
}

impl StatusLineAccent {
    fn for_item(item: StatusLineItem) -> Self {
        match item {
            StatusLineItem::ModelName
            | StatusLineItem::ModelWithReasoning
            | StatusLineItem::Reasoning => Self::Model,
            StatusLineItem::CurrentDir | StatusLineItem::ProjectRoot => Self::Path,
            StatusLineItem::GitBranch
            | StatusLineItem::PullRequestNumber
            | StatusLineItem::BranchChanges => Self::Branch,
            StatusLineItem::Status => Self::State,
            StatusLineItem::ContextRemaining
            | StatusLineItem::ContextUsed
            | StatusLineItem::ContextTokens
            | StatusLineItem::ContextWindowSize
            | StatusLineItem::UsedTokens
            | StatusLineItem::TotalInputTokens
            | StatusLineItem::TotalOutputTokens
            | StatusLineItem::ThreadCredits
            | StatusLineItem::EstimatedThreadCost => Self::Usage,
            StatusLineItem::ContextProgress => Self::Progress,
            StatusLineItem::SessionTiming => Self::Metadata,
            StatusLineItem::FiveHourLimit | StatusLineItem::WeeklyLimit | StatusLineItem::Quota => {
                Self::Limit
            }
            StatusLineItem::CodexVersion | StatusLineItem::Hostname | StatusLineItem::SessionId => {
                Self::Metadata
            }
            StatusLineItem::FastMode | StatusLineItem::RawOutput => Self::Mode,
            StatusLineItem::Permissions => Self::Mode,
            StatusLineItem::ApprovalMode => Self::Mode,
            StatusLineItem::ThreadName
            | StatusLineItem::ThreadTitle
            | StatusLineItem::WorkspaceHeadline => Self::Thread,
            StatusLineItem::TaskProgress => Self::Progress,
        }
    }

    fn scopes(self) -> &'static [&'static str] {
        match self {
            Self::Model => &["entity.name.type", "support.type", "variable"],
            Self::Path => &["string", "markup.underline.link"],
            Self::Branch => &["entity.name.function", "entity.name.tag"],
            Self::State => &["keyword.control", "keyword"],
            Self::Usage => &["constant.numeric", "constant"],
            Self::Limit => &["constant.language", "storage.type"],
            Self::Metadata => &["comment", "constant.other"],
            Self::Mode => &["storage.modifier", "keyword.operator"],
            Self::Thread => &["markup.heading", "entity.name.section"],
            Self::Progress => &["markup.inserted", "constant.numeric"],
        }
    }

    fn fallback_style(self) -> Style {
        match self {
            Self::Model | Self::State | Self::Metadata | Self::Mode => Style::default().cyan(),
            Self::Path | Self::Usage | Self::Progress => Style::default().green(),
            Self::Branch | Self::Limit | Self::Thread => Style::default().magenta(),
        }
    }

    fn ccu_role(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Usage => "usage",
            Self::Progress => "progress",
            Self::Limit => "quota",
            Self::State | Self::Mode | Self::Metadata => "time",
            Self::Path | Self::Branch | Self::Thread => "usage",
        }
    }
}

pub(crate) fn status_line_from_segments<I>(
    segments: I,
    use_theme_colors: bool,
    thread_id: Option<ThreadId>,
) -> Option<Line<'static>>
where
    I: IntoIterator<Item = (StatusLineItem, String)>,
{
    let ccu_theme = use_theme_colors.then(crate::ccu_theme::active).flatten();
    status_line_from_segments_with_theme(
        segments,
        use_theme_colors,
        thread_id,
        |accent| foreground_style_for_scopes(accent.scopes()),
        ccu_theme,
    )
}

#[cfg(test)]
fn status_line_from_segments_with_resolver<I, F>(
    segments: I,
    use_theme_colors: bool,
    thread_id: Option<ThreadId>,
    theme_style_for_accent: F,
) -> Option<Line<'static>>
where
    I: IntoIterator<Item = (StatusLineItem, String)>,
    F: Fn(StatusLineAccent) -> Option<Style>,
{
    status_line_from_segments_with_theme(
        segments,
        use_theme_colors,
        thread_id,
        theme_style_for_accent,
        None,
    )
}

fn status_line_from_segments_with_theme<I, F>(
    segments: I,
    use_theme_colors: bool,
    thread_id: Option<ThreadId>,
    theme_style_for_accent: F,
    ccu_theme: Option<&crate::ccu_theme::CcuTheme>,
) -> Option<Line<'static>>
where
    I: IntoIterator<Item = (StatusLineItem, String)>,
    F: Fn(StatusLineAccent) -> Option<Style>,
{
    let mut spans = Vec::new();
    let separator = ccu_theme
        .map(super::super::ccu_theme::CcuTheme::separator)
        .unwrap_or(STATUS_LINE_SEPARATOR);
    for (item, text) in segments {
        if !spans.is_empty() {
            spans.push(
                ccu_theme
                    .and_then(|theme| theme.status_style("separator"))
                    .map_or_else(
                        || Span::from(separator.to_string()).set_style(secondary_text_style()),
                        |style| Span::styled(separator.to_string(), style),
                    ),
            );
        }
        let style = if use_theme_colors
            && matches!(
                item,
                StatusLineItem::ThreadName | StatusLineItem::ThreadTitle
            )
            && let Some(thread_id) = thread_id
        {
            Style::default().fg(thread_color(thread_id))
        } else if use_theme_colors {
            let accent = StatusLineAccent::for_item(item);
            let style = ccu_theme
                .and_then(|theme| theme.status_style(accent.ccu_role()))
                .or_else(|| theme_style_for_accent(accent))
                .unwrap_or_else(|| accent.fallback_style());
            if ccu_theme.is_none_or(crate::ccu_theme::CcuTheme::soften_status_line_colors) {
                soften_status_line_style(style)
            } else {
                style
            }
        } else {
            secondary_text_style()
        };
        let style = if use_theme_colors {
            style.fg(readable_color_on(
                style.fg.unwrap_or(Color::Reset),
                /*background*/ None,
            ))
        } else {
            style
        };
        let style = if item == StatusLineItem::PullRequestNumber {
            style.underlined()
        } else {
            style
        };
        if let Some(theme) = ccu_theme.filter(|theme| theme.claude_layout()) {
            if item == StatusLineItem::EstimatedThreadCost {
                spans.push(Span::styled(
                    text,
                    theme.status_style("quota").unwrap_or(style),
                ));
                continue;
            }
            if item == StatusLineItem::ContextProgress
                && let Some(percent) = text
                    .rsplit_once(' ')
                    .and_then(|(_, value)| value.strip_suffix('%'))
                    .and_then(|value| value.parse::<u8>().ok())
            {
                spans.extend(theme.progress_spans(percent));
                continue;
            }
            if item == StatusLineItem::SessionTiming
                && let Some((elapsed, active)) = text.split_once('⚡')
            {
                spans.push(Span::styled(elapsed.to_string(), style));
                spans.push(Span::styled(
                    format!("⚡{active}"),
                    theme.status_style("activeTime").unwrap_or(style),
                ));
                continue;
            }
            if matches!(
                item,
                StatusLineItem::Permissions | StatusLineItem::ApprovalMode
            ) {
                spans.push(Span::styled(
                    text,
                    theme.status_style("permissions").unwrap_or(style),
                ));
                continue;
            }
        }
        spans.push(Span::styled(text, style));
    }

    (!spans.is_empty()).then(|| Line::from(spans))
}

/// Fits a structured status line by removing complete trailing segments before truncating text.
///
/// Values can contain several styled spans. Remove at separator boundaries to avoid
/// half-visible progress bars or timers on narrow terminals. The default CCU order
/// intentionally places progressively less essential items to the right.
pub(crate) fn fit_status_line_to_width(mut line: Line<'static>, max_width: usize) -> Line<'static> {
    if line.width() <= max_width {
        return line;
    }

    let separator = crate::ccu_theme::active()
        .map(crate::ccu_theme::CcuTheme::separator)
        .unwrap_or(" │ ");
    while let Some(index) = line
        .spans
        .iter()
        .rposition(|span| span.content == separator || span.content == STATUS_LINE_SEPARATOR)
    {
        line.spans.truncate(index);
        if line.width() <= max_width {
            return line;
        }
    }

    truncate_line_with_ellipsis_if_overflow(line, max_width)
}

fn soften_status_line_style(mut style: Style) -> Style {
    if let Some(fg) = style.fg {
        style.fg = Some(soften_status_line_color(fg));
    }
    style
}

#[allow(clippy::disallowed_methods)]
fn soften_status_line_color(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let luma = weighted_luma(r, g, b);
            Color::Rgb(
                soften_rgb_channel(r, luma),
                soften_rgb_channel(g, luma),
                soften_rgb_channel(b, luma),
            )
        }
        Color::LightRed => Color::Red,
        Color::LightGreen => Color::Green,
        Color::LightYellow => Color::Yellow,
        Color::LightBlue => Color::Blue,
        Color::LightMagenta => Color::Magenta,
        Color::LightCyan => Color::Cyan,
        Color::White => Color::Gray,
        Color::Reset
        | Color::Black
        | Color::Red
        | Color::Green
        | Color::Yellow
        | Color::Blue
        | Color::Magenta
        | Color::Cyan
        | Color::Gray
        | Color::DarkGray
        | Color::Indexed(_) => color,
    }
}

fn weighted_luma(r: u8, g: u8, b: u8) -> u16 {
    (77 * u16::from(r) + 150 * u16::from(g) + 29 * u16::from(b)) / 256
}

fn soften_rgb_channel(channel: u8, luma: u16) -> u8 {
    let channel = u16::from(channel);
    let softened = (channel * STATUS_LINE_COLOR_SATURATION_PERCENT
        + luma * (100 - STATUS_LINE_COLOR_SATURATION_PERCENT)
        + 50)
        / 100;

    ((softened * STATUS_LINE_COLOR_BRIGHTNESS_PERCENT + 50) / 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Modifier;
    use ratatui::style::Stylize;

    fn line_text(line: &Line<'static>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn status_line_segments_preserve_order_and_plain_text() {
        let line = status_line_from_segments_with_resolver(
            [
                (StatusLineItem::ModelName, "gpt-5".to_string()),
                (StatusLineItem::CurrentDir, "/repo".to_string()),
                (StatusLineItem::GitBranch, "main".to_string()),
            ],
            /*use_theme_colors*/ true,
            /*thread_id*/ None,
            |_| None,
        )
        .expect("status line");

        assert_eq!(line_text(&line), "gpt-5 · /repo · main");
        assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
        assert!(!line.spans[0].style.add_modifier.contains(Modifier::DIM));
        assert_eq!(line.spans[2].style.fg, Some(Color::Green));
        assert!(!line.spans[2].style.add_modifier.contains(Modifier::DIM));
        assert_eq!(line.spans[4].style.fg, Some(Color::Magenta));
        assert!(!line.spans[4].style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn status_line_segments_use_secondary_separators_and_theme_styles_first() {
        let line = status_line_from_segments_with_resolver(
            [
                (StatusLineItem::ModelName, "gpt-5".to_string()),
                (StatusLineItem::ContextUsed, "Context 12% used".to_string()),
            ],
            /*use_theme_colors*/ true,
            /*thread_id*/ None,
            |accent| match accent {
                StatusLineAccent::Model => Some(Style::default().red()),
                _ => None,
            },
        )
        .expect("status line");

        assert_eq!(line.spans[0].style.fg, Some(Color::Red));
        assert!(!line.spans[0].style.add_modifier.contains(Modifier::DIM));
        assert_eq!(line.spans[1].style, secondary_text_style());
        assert_eq!(line.spans[2].style.fg, Some(Color::Green));
        assert!(!line.spans[2].style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn thread_usage_items_share_an_accent_and_secondary_separator() {
        let line = status_line_from_segments_with_resolver(
            [
                (StatusLineItem::ThreadCredits, "5.2 credits".to_string()),
                (StatusLineItem::EstimatedThreadCost, "~$0.21".to_string()),
            ],
            /*use_theme_colors*/ true,
            /*thread_id*/ None,
            |_| None,
        )
        .expect("thread usage status line");

        assert_eq!(line_text(&line), "5.2 credits · ~$0.21");
        assert_eq!(line.spans[0].style, line.spans[2].style);
        assert_eq!(line.spans[1].style, secondary_text_style());
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn status_line_segments_soften_rgb_theme_styles_without_dimming_text() {
        let line = status_line_from_segments_with_resolver(
            [(StatusLineItem::ModelName, "gpt-5".to_string())],
            /*use_theme_colors*/ true,
            /*thread_id*/ None,
            |_| Some(Style::default().fg(Color::Rgb(255, 0, 0))),
        )
        .expect("status line");

        assert_eq!(
            line.spans[0].style.fg,
            Some(readable_color_on(
                Color::Rgb(228, 11, 11),
                /*background*/ None
            ))
        );
        assert!(!line.spans[0].style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn status_line_segments_can_disable_theme_colors() {
        let line = status_line_from_segments_with_resolver(
            [
                (StatusLineItem::ModelName, "gpt-5".to_string()),
                (StatusLineItem::ContextUsed, "Context 12% used".to_string()),
            ],
            /*use_theme_colors*/ false,
            /*thread_id*/ None,
            |_| Some(Style::default().red()),
        )
        .expect("status line");

        assert_eq!(line_text(&line), "gpt-5 · Context 12% used");
        assert_eq!(line.spans[0].style, secondary_text_style());
        assert_eq!(line.spans[1].style, secondary_text_style());
        assert_eq!(line.spans[2].style, secondary_text_style());
    }

    #[test]
    fn pull_request_number_uses_link_style() {
        let line = status_line_from_segments_with_resolver(
            [(StatusLineItem::PullRequestNumber, "PR #20252".to_string())],
            /*use_theme_colors*/ false,
            /*thread_id*/ None,
            |_| None,
        )
        .expect("status line");

        assert_eq!(line.spans[0].style, secondary_text_style().underlined());
    }

    #[test]
    fn status_line_segments_return_none_when_empty() {
        assert_eq!(
            status_line_from_segments_with_resolver(
                Vec::<(StatusLineItem, String)>::new(),
                /*use_theme_colors*/ true,
                /*thread_id*/ None,
                |_| None,
            ),
            None
        );
    }

    #[test]
    fn light_status_line_corrects_pale_custom_theme_colors() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let colors = crate::terminal_probe::DefaultColors {
            fg: (30, 30, 30),
            bg: (255, 255, 255),
        };
        crate::terminal_palette::with_test_default_colors(colors, || {
            let line = status_line_from_segments_with_resolver(
                [
                    (StatusLineItem::ModelName, "gpt-5".to_string()),
                    (StatusLineItem::CurrentDir, "~/code".to_string()),
                    (
                        StatusLineItem::Permissions,
                        "Custom permissions".to_string(),
                    ),
                ],
                /*use_theme_colors*/ true,
                /*thread_id*/ None,
                |accent| {
                    let rgb = match accent {
                        StatusLineAccent::Model => (240, 210, 160),
                        StatusLineAccent::Path => (190, 230, 180),
                        _ => (215, 180, 240),
                    };
                    Some(Style::default().fg(crate::terminal_palette::rgb_color(rgb)))
                },
            )
            .expect("status line");
            let area = Rect::new(
                /*x*/ 0, /*y*/ 0, /*width*/ 42, /*height*/ 1,
            );
            let mut buffer = Buffer::empty(area);
            line.render(area, &mut buffer);
            insta::assert_snapshot!(format!("{buffer:?}"));
        });
    }

    #[test]
    fn responsive_status_line_drops_complete_trailing_segments() {
        let line = Line::from(vec![
            "🦊 gpt-5.6-sol[xhigh]".cyan(),
            " │ ".dim(),
            "42.7K/353K".green(),
            " │ ".dim(),
            "[█░░░░░░░░░] 9%".green(),
            " │ ".dim(),
            "⏱ 1s ⚡0s".cyan(),
        ]);

        let rendered = [72, 60, 40, 16]
            .into_iter()
            .map(|width| {
                let fitted = fit_status_line_to_width(line.clone(), width);
                format!("{width:>2}: {}", line_text(&fitted))
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!("ccu_status_line_responsive_widths", rendered);
    }

    #[test]
    fn claude_status_line_preserves_colored_segments_at_narrow_widths() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;
        let theme = crate::ccu_theme::parse_theme(
            include_str!("../../assets/ccu-claude-theme.json"),
            "rainbow_color",
            1,
        )
        .expect("theme");
        let line = status_line_from_segments_with_theme(
            [
                (
                    StatusLineItem::ModelWithReasoning,
                    "🐱 gpt-6-astra[high]".to_string(),
                ),
                (StatusLineItem::ContextTokens, "60.3K/1M".to_string()),
                (StatusLineItem::ContextProgress, theme.progress(6)),
                (StatusLineItem::SessionTiming, "⏱ 18s ⚡8s".to_string()),
                (StatusLineItem::Quota, "15.00 CNY".to_string()),
            ],
            true,
            None,
            |_| None,
            Some(&theme),
        )
        .expect("line");
        let mut buffer = Buffer::empty(Rect::new(0, 0, 120, 4));
        for (y, width) in [120, 80, 55, 30].into_iter().enumerate() {
            let fitted = fit_status_line_to_width(line.clone(), width);
            let text = fitted.to_string();
            assert!(!text.contains('[') || text.contains(']'));
            assert!(!text.contains('⏱') || text.contains("⚡8s"));
            assert!(!text.ends_with(" │ "));
            fitted.render(Rect::new(0, y as u16, width as u16, 1), &mut buffer);
        }
        insta::assert_snapshot!(format!("{buffer:?}"));
    }
}
