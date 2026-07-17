use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use codex_utils_home_dir::find_codex_home;
use ratatui::style::Color;
use ratatui::style::Style;
use serde::Deserialize;

const DEFAULT_THEME_ID: &str = "ccu.deepseek";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeDocument {
    schema_version: u32,
    #[serde(rename = "type")]
    kind: String,
    id: String,
    status_line: StatusLineTheme,
    welcome: WelcomeTheme,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusLineTheme {
    separator: String,
    progress_width: usize,
    filled: String,
    empty: String,
    colors: StatusLineColors,
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
}

#[derive(Clone, Debug)]
pub(crate) struct CcuTheme {
    status_line: StatusLineTheme,
    welcome: WelcomeTheme,
}

static THEME: OnceLock<Option<CcuTheme>> = OnceLock::new();

pub(crate) fn active() -> Option<&'static CcuTheme> {
    THEME.get_or_init(load_theme).as_ref()
}

fn load_theme() -> Option<CcuTheme> {
    let codex_home = find_codex_home().ok()?;
    let theme_id = std::env::var("CODEX_CCU_THEME")
        .ok()
        .filter(|value| !value.trim().is_empty())
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
    let document: ThemeDocument = serde_json::from_str(&source).ok()?;
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
        parse_color(value)?;
    }
    Some(CcuTheme {
        status_line: document.status_line,
        welcome: document.welcome,
    })
}

fn is_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

fn parse_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(red, green, blue))
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
        parse_color(value).map(|color| Style::default().fg(color))
    }

    pub(crate) fn welcome_style(&self, role: &str) -> Option<Style> {
        let value = match role {
            "title" => &self.welcome.title,
            "version" => &self.welcome.version,
            "label" => &self.welcome.label,
            "model" => &self.welcome.model,
            "path" => &self.welcome.path,
            "permissions" => &self.welcome.permissions,
            _ => return None,
        };
        parse_color(value).map(|color| Style::default().fg(color))
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

    #[test]
    fn parses_rgb_colors() {
        assert_eq!(parse_color("#5eead4"), Some(Color::Rgb(94, 234, 212)));
        assert_eq!(parse_color("cyan"), None);
    }

    #[test]
    fn default_progress_has_ten_cells() {
        assert_eq!(render_progress(0), "[░░░░░░░░░░] 0%");
        assert_eq!(render_progress(50), "[█████░░░░░] 50%");
    }
}
