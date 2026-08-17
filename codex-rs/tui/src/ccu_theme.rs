use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_utils_home_dir::find_codex_home;
use ratatui::style::Color;
use ratatui::style::Style;
use serde::Deserialize;

use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::best_color_for_level;
use crate::terminal_palette::rgb_color;
use crate::terminal_palette::stdout_color_level;

const DEFAULT_THEME_ID: &str = "rainbow_color";
const LEGACY_THEME_ID: &str = "ccu.deepseek";
const STATUS_LINE_PRESET_FILE: &str = "ui-statusline-preset";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeDocument {
    schema_version: u32,
    #[serde(rename = "type")]
    kind: String,
    id: String,
    status_line: StatusLineTheme,
    welcome: WelcomeTheme,
    #[serde(default)]
    status_card: Option<StatusCardTheme>,
    #[serde(default)]
    dialog: Option<DialogTheme>,
    #[serde(default)]
    composer: Option<SurfaceTheme>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusLineTheme {
    separator: String,
    progress_width: usize,
    filled: String,
    empty: String,
    #[serde(default)]
    model_emojis: Vec<String>,
    #[serde(default)]
    palette: Vec<String>,
    #[serde(default = "default_true")]
    randomize_palette: bool,
    #[serde(default = "default_true")]
    soften_colors: bool,
    #[serde(default)]
    model_reasoning_style: ModelReasoningStyle,
    colors: StatusLineColors,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ModelReasoningStyle {
    #[default]
    Spaced,
    Bracketed,
}

#[derive(Clone, Debug, Deserialize)]
struct StatusLineColors {
    model: String,
    usage: String,
    progress: String,
    time: String,
    quota: String,
    separator: String,
}

#[derive(Clone, Debug, Deserialize)]
struct WelcomeTheme {
    title: String,
    version: String,
    label: String,
    model: String,
    path: String,
    permissions: String,
    #[serde(default)]
    border: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    badge: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct StatusCardTheme {
    border: String,
    title: String,
    version: String,
    label: String,
    model: String,
    path: String,
    permissions: String,
    usage: String,
    progress: String,
    percent: String,
    limits: String,
    link: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DialogTheme {
    selection: String,
    background: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct SurfaceTheme {
    background: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CcuTheme {
    status_line: StatusLineTheme,
    welcome: WelcomeTheme,
    status_card: Option<StatusCardTheme>,
    dialog: Option<DialogTheme>,
    composer: Option<SurfaceTheme>,
    model_emoji: Option<String>,
}

static THEME: OnceLock<Option<CcuTheme>> = OnceLock::new();

pub(crate) fn active() -> Option<&'static CcuTheme> {
    THEME.get_or_init(load_theme).as_ref()
}

pub(crate) fn status_line_preset_enabled(codex_home: &Path) -> bool {
    status_line_preset(codex_home).is_some()
}

fn status_line_preset(codex_home: &Path) -> Option<String> {
    fs::read_to_string(codex_home.join(STATUS_LINE_PRESET_FILE))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| is_safe_id(value))
        .map(|value| {
            if value == LEGACY_THEME_ID {
                DEFAULT_THEME_ID.to_string()
            } else {
                value
            }
        })
}

fn load_theme() -> Option<CcuTheme> {
    let codex_home = find_codex_home().ok()?;
    let theme_id = std::env::var("CODEX_CCU_THEME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| status_line_preset(&codex_home))
        .or_else(|| {
            fs::read_to_string(codex_home.join("ui-theme"))
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| DEFAULT_THEME_ID.to_string());
    if !is_safe_id(&theme_id) {
        return None;
    }
    let root = std::env::var_os("CODEX_CCU_THEME_PACK_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| codex_home.join("ccu").join("themes").to_path_buf());
    let source = fs::read_to_string(root.join(&theme_id).join("theme.json")).ok()?;
    parse_theme(&source, &theme_id, session_seed())
}

fn parse_theme(source: &str, theme_id: &str, seed: u64) -> Option<CcuTheme> {
    let mut document: ThemeDocument = serde_json::from_str(source).ok()?;
    if document.schema_version != 1
        || document.kind != "theme"
        || document.id != theme_id
        || document.status_line.separator.is_empty()
        || !(4..=30).contains(&document.status_line.progress_width)
        || document.status_line.filled.is_empty()
        || document.status_line.empty.is_empty()
    {
        return None;
    }
    for value in [
        &document.status_line.colors.model,
        &document.status_line.colors.usage,
        &document.status_line.colors.progress,
        &document.status_line.colors.time,
        &document.status_line.colors.quota,
        &document.status_line.colors.separator,
        &document.welcome.title,
        &document.welcome.version,
        &document.welcome.label,
        &document.welcome.model,
        &document.welcome.path,
        &document.welcome.permissions,
    ] {
        parse_rgb(value)?;
    }
    for value in [
        document.welcome.border.as_deref(),
        document.welcome.command.as_deref(),
        document.welcome.badge.as_deref(),
        document
            .dialog
            .as_ref()
            .map(|theme| theme.selection.as_str()),
        document
            .dialog
            .as_ref()
            .and_then(|theme| theme.background.as_deref()),
        document
            .composer
            .as_ref()
            .and_then(|theme| theme.background.as_deref()),
    ]
    .into_iter()
    .flatten()
    {
        parse_rgb(value)?;
    }
    if let Some(theme) = document.status_card.as_ref() {
        for value in [
            &theme.border,
            &theme.title,
            &theme.version,
            &theme.label,
            &theme.model,
            &theme.path,
            &theme.permissions,
            &theme.usage,
            &theme.progress,
            &theme.percent,
            &theme.limits,
            &theme.link,
            &theme.value,
        ] {
            parse_rgb(value)?;
        }
    }
    for value in &document.status_line.palette {
        parse_rgb(value)?;
    }
    if document.status_line.model_emojis.iter().any(|emoji| {
        emoji.is_empty() || emoji.chars().count() > 8 || emoji.chars().any(char::is_control)
    }) {
        return None;
    }
    if document.status_line.randomize_palette {
        apply_palette(
            &mut document.status_line.colors,
            &document.status_line.palette,
            seed,
        );
    }
    let model_emoji = select_model_emoji(&document.status_line.model_emojis, seed);
    Some(CcuTheme {
        status_line: document.status_line,
        welcome: document.welcome,
        status_card: document.status_card,
        dialog: document.dialog,
        composer: document.composer,
        model_emoji,
    })
}

const fn default_true() -> bool {
    true
}

fn session_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    nanos ^ u64::from(std::process::id())
}

fn next_random(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 0x9e37_79b9_7f4a_7c15;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn shuffled_palette(palette: &[String], mut seed: u64) -> Vec<String> {
    let mut colors = palette.to_vec();
    for index in (1..colors.len()).rev() {
        let swap_index = (next_random(&mut seed) as usize) % (index + 1);
        colors.swap(index, swap_index);
    }
    colors
}

fn apply_palette(colors: &mut StatusLineColors, palette: &[String], seed: u64) {
    let shuffled = shuffled_palette(palette, seed);
    if shuffled.len() < 6 {
        return;
    }
    colors.model = shuffled[0].clone();
    colors.separator = shuffled[1].clone();
    colors.usage = shuffled[2].clone();
    colors.progress = shuffled[3].clone();
    colors.time = shuffled[4].clone();
    colors.quota = shuffled[5].clone();
}

fn select_model_emoji(emojis: &[String], seed: u64) -> Option<String> {
    (!emojis.is_empty()).then(|| emojis[(seed as usize) % emojis.len()].clone())
}

fn is_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn parse_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((red, green, blue))
}

fn theme_color(value: &str) -> Option<Color> {
    theme_color_with(value, |rgb| {
        theme_color_for_level(rgb, stdout_color_level(), cfg!(windows))
    })
}

fn theme_color_for_level(rgb: (u8, u8, u8), color_level: StdoutColorLevel, windows: bool) -> Color {
    if windows
        && matches!(
            color_level,
            StdoutColorLevel::Ansi16 | StdoutColorLevel::Unknown
        )
    {
        return rgb_color(rgb);
    }
    best_color_for_level(rgb, color_level)
}

fn theme_color_with(value: &str, resolve: impl FnOnce((u8, u8, u8)) -> Color) -> Option<Color> {
    let color = resolve(parse_rgb(value)?);
    (color != Color::Reset).then_some(color)
}

fn format_model_reasoning(
    style: ModelReasoningStyle,
    model: &str,
    reasoning: &str,
    service_tier: Option<&str>,
) -> String {
    match style {
        ModelReasoningStyle::Spaced => service_tier.map_or_else(
            || format!("{model} {reasoning}"),
            |service_tier| format!("{model} {reasoning} {service_tier}"),
        ),
        ModelReasoningStyle::Bracketed => {
            let labels = service_tier.map_or_else(
                || reasoning.to_string(),
                |service_tier| format!("{reasoning},{service_tier}"),
            );
            format!("{model}[{labels}]")
        }
    }
}

impl CcuTheme {
    pub(crate) fn separator(&self) -> &str {
        &self.status_line.separator
    }

    pub(crate) fn status_style(&self, role: &str) -> Option<Style> {
        let value = match role {
            "model" => &self.status_line.colors.model,
            "usage" => &self.status_line.colors.usage,
            "progress" => &self.status_line.colors.progress,
            "time" => &self.status_line.colors.time,
            "quota" => &self.status_line.colors.quota,
            "separator" => &self.status_line.colors.separator,
            _ => return None,
        };
        theme_color(value).map(|color| Style::default().fg(color))
    }

    pub(crate) fn soften_status_line_colors(&self) -> bool {
        self.status_line.soften_colors
    }

    pub(crate) fn welcome_style(&self, role: &str) -> Option<Style> {
        let value = match role {
            "title" => &self.welcome.title,
            "version" => &self.welcome.version,
            "label" => &self.welcome.label,
            "model" => &self.welcome.model,
            "path" => &self.welcome.path,
            "permissions" => &self.welcome.permissions,
            "border" => self.welcome.border.as_ref()?,
            "command" => self.welcome.command.as_ref()?,
            "badge" => self.welcome.badge.as_ref()?,
            _ => return None,
        };
        theme_color(value).map(|color| Style::default().fg(color))
    }

    pub(crate) fn status_card_style(&self, role: &str) -> Option<Style> {
        let theme = self.status_card.as_ref()?;
        let value = match role {
            "border" => &theme.border,
            "title" => &theme.title,
            "version" => &theme.version,
            "label" => &theme.label,
            "model" => &theme.model,
            "path" => &theme.path,
            "permissions" => &theme.permissions,
            "usage" => &theme.usage,
            "progress" => &theme.progress,
            "percent" => &theme.percent,
            "limits" => &theme.limits,
            "link" => &theme.link,
            "value" => &theme.value,
            _ => return None,
        };
        theme_color(value).map(|color| Style::default().fg(color))
    }

    pub(crate) fn dialog_selection_style(&self) -> Option<Style> {
        theme_color(&self.dialog.as_ref()?.selection).map(|color| Style::default().fg(color))
    }

    pub(crate) fn dialog_surface_style(&self) -> Option<Style> {
        self.dialog
            .as_ref()
            .map(|theme| surface_style(theme.background.as_deref()))
    }

    pub(crate) fn composer_style(&self) -> Option<Style> {
        self.composer
            .as_ref()
            .map(|theme| surface_style(theme.background.as_deref()))
    }

    pub(crate) fn progress(&self, used_percent: u8) -> String {
        let ratio = f64::from(used_percent.clamp(0, 100)) / 100.0;
        let filled = (ratio * self.status_line.progress_width as f64).round() as usize;
        let filled = filled.min(self.status_line.progress_width);
        let empty = self.status_line.progress_width.saturating_sub(filled);
        format!(
            "[{}{}] {}%",
            self.status_line.filled.repeat(filled),
            self.status_line.empty.repeat(empty),
            used_percent
        )
    }

    fn format_model_with_reasoning(
        &self,
        model: &str,
        reasoning: &str,
        service_tier: Option<&str>,
    ) -> String {
        let label = format_model_reasoning(
            self.status_line.model_reasoning_style,
            model,
            reasoning,
            service_tier,
        );
        self.model_emoji
            .as_deref()
            .map_or(label.clone(), |emoji| format!("{emoji} {label}"))
    }
}

fn surface_style(background: Option<&str>) -> Style {
    background
        .and_then(theme_color)
        .map_or_else(Style::default, |color| Style::default().bg(color))
}

pub(crate) fn format_status_line_model(
    model: &str,
    reasoning: &str,
    service_tier: Option<&str>,
) -> String {
    active().map_or_else(
        || format_model_reasoning(ModelReasoningStyle::Spaced, model, reasoning, service_tier),
        |theme| theme.format_model_with_reasoning(model, reasoning, service_tier),
    )
}

pub(crate) fn render_progress(used_percent: u8) -> String {
    active().map_or_else(
        || {
            let width = 10;
            let filled = ((usize::from(used_percent.clamp(0, 100)) * width) + 50) / 100;
            format!(
                "[{}{}] {}%",
                "█".repeat(filled),
                "░".repeat(width.saturating_sub(filled)),
                used_percent
            )
        },
        |theme| theme.progress(used_percent),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAINBOW_THEME: &str = r##"{
        "schemaVersion": 1,
        "type": "theme",
        "id": "rainbow_color",
        "statusLine": {
            "separator": " | ",
            "progressWidth": 10,
            "filled": "#",
            "empty": "-",
            "modelEmojis": ["A", "B"],
            "palette": ["#F5E0DC", "#F2CDCD", "#F5C2E7", "#FAB387", "#F9E2AF", "#A6E3A1"],
            "randomizePalette": false,
            "softenColors": false,
            "colors": {
                "model": "#F5E0DC",
                "usage": "#F5C2E7",
                "progress": "#A6E3A1",
                "time": "#F9E2AF",
                "quota": "#FAB387",
                "separator": "#CBA6F7"
            }
        },
        "welcome": {
            "title": "#89DCEB",
            "version": "#F5E0DC",
            "label": "#89DCEB",
            "model": "#F2CDCD",
            "path": "#A6E3A1",
            "permissions": "#FAB387",
            "border": "#89DCEB",
            "command": "#89DCEB",
            "badge": "#F5C2E7"
        },
        "statusCard": {
            "border": "#89DCEB",
            "title": "#89DCEB",
            "version": "#F5E0DC",
            "label": "#89DCEB",
            "model": "#F2CDCD",
            "path": "#A6E3A1",
            "permissions": "#FAB387",
            "usage": "#89DCEB",
            "progress": "#A6E3A1",
            "percent": "#74C7EC",
            "limits": "#F9E2AF",
            "link": "#89DCEB",
            "value": "#CDD6F4"
        },
        "dialog": { "selection": "#89DCEB", "background": null },
        "composer": { "background": null }
    }"##;

    #[test]
    fn parses_rgb_colors() {
        assert_eq!(parse_rgb("#5eead4"), Some((94, 234, 212)));
        assert_eq!(parse_rgb("cyan"), None);
    }

    #[test]
    fn safe_theme_ids_allow_rainbow_color() {
        assert!(is_safe_id("rainbow_color"));
    }

    #[test]
    fn formats_compact_model_reasoning_labels() {
        assert_eq!(
            format_model_reasoning(ModelReasoningStyle::Bracketed, "gpt-5.6-sol", "xhigh", None,),
            "gpt-5.6-sol[xhigh]"
        );
        assert_eq!(
            format_model_reasoning(
                ModelReasoningStyle::Bracketed,
                "gpt-5.4",
                "xhigh",
                Some("fast"),
            ),
            "gpt-5.4[xhigh,fast]"
        );
    }

    #[test]
    fn session_seed_selects_a_stable_emoji_and_palette_order() {
        let emojis = vec!["🦊".to_string(), "🚀".to_string(), "🌈".to_string()];
        assert_eq!(select_model_emoji(&emojis, 4).as_deref(), Some("🚀"));

        let palette = vec![
            "#F5E0DC".to_string(),
            "#F2CDCD".to_string(),
            "#F5C2E7".to_string(),
            "#FAB387".to_string(),
            "#F9E2AF".to_string(),
            "#A6E3A1".to_string(),
        ];
        assert_eq!(
            shuffled_palette(&palette, 42),
            shuffled_palette(&palette, 42)
        );
    }

    #[test]
    fn theme_colors_fall_back_when_the_terminal_has_no_rich_color_support() {
        assert_eq!(
            theme_color_with("#5eead4", |rgb| {
                crate::terminal_palette::best_color_for_level(
                    rgb,
                    crate::terminal_palette::StdoutColorLevel::Ansi16,
                )
            }),
            None
        );
        assert!(matches!(
            theme_color_with("#5eead4", |rgb| {
                crate::terminal_palette::best_color_for_level(
                    rgb,
                    crate::terminal_palette::StdoutColorLevel::Ansi256,
                )
            }),
            Some(Color::Indexed(_))
        ));
    }

    #[test]
    fn rainbow_theme_keeps_role_colors_fixed_but_varies_emoji() {
        let first = parse_theme(RAINBOW_THEME, "rainbow_color", 1).expect("valid theme");
        let second = parse_theme(RAINBOW_THEME, "rainbow_color", 2).expect("valid theme");

        assert_eq!(
            first.status_line.colors.model,
            second.status_line.colors.model
        );
        assert_ne!(first.model_emoji, second.model_emoji);
        assert!(!first.soften_status_line_colors());
        assert_eq!(first.dialog_surface_style(), Some(Style::default()));
        assert_eq!(first.composer_style(), Some(Style::default()));
        insta::assert_debug_snapshot!("rainbow_color_theme_roles", first);
    }

    #[test]
    fn windows_ccu_theme_promotes_legacy_color_levels_to_rgb() {
        assert_eq!(
            theme_color_for_level((137, 220, 235), StdoutColorLevel::Ansi16, true),
            rgb_color((137, 220, 235))
        );
        assert_eq!(
            theme_color_for_level((137, 220, 235), StdoutColorLevel::Unknown, true),
            rgb_color((137, 220, 235))
        );
    }

    #[test]
    fn default_progress_has_ten_cells() {
        assert_eq!(render_progress(0), "[░░░░░░░░░░] 0%");
        assert_eq!(render_progress(50), "[█████░░░░░] 50%");
    }

    #[test]
    fn status_line_preset_requires_an_explicit_safe_preference() {
        let codex_home = tempfile::tempdir().expect("temp codex home");
        assert!(!status_line_preset_enabled(codex_home.path()));

        fs::write(
            codex_home.path().join(STATUS_LINE_PRESET_FILE),
            "rainbow_color\n",
        )
        .expect("write CCU status-line preference");
        assert!(status_line_preset_enabled(codex_home.path()));

        fs::write(
            codex_home.path().join(STATUS_LINE_PRESET_FILE),
            "ccu.deepseek\n",
        )
        .expect("write legacy CCU status-line preference");
        assert!(status_line_preset_enabled(codex_home.path()));

        fs::write(
            codex_home.path().join(STATUS_LINE_PRESET_FILE),
            "future.theme",
        )
        .expect("write unknown status-line preference");
        assert!(status_line_preset_enabled(codex_home.path()));
    }
}
