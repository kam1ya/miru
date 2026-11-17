use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const PRESETS: &[&str] = &[
    "miru",
    "catppuccin",
    "tokyo-night",
    "gruvbox",
    "dracula",
    "nord",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSection {
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_fg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_warn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_crit: Option<String>,
}

fn default_preset() -> String {
    "miru".to_string()
}

impl Default for ThemeSection {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            border: None,
            primary: None,
            secondary: None,
            highlight_bg: None,
            highlight_fg: None,
            text_warn: None,
            text_crit: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedTheme {
    pub border: String,
    pub primary: String,
    pub secondary: String,
    pub highlight_bg: String,
    pub highlight_fg: String,
    pub text_warn: String,
    pub text_crit: String,
}

impl ResolvedTheme {
    pub fn from_preset(name: &str) -> Self {
        let (border, primary, secondary, hl_bg, hl_fg, warn, crit) =
            match name.to_lowercase().as_str() {
                "catppuccin" => (
                    "#45475a", "#89b4fa", "#f9e2af", "#313244", "#cba6f7", "#fab387", "#f38ba8",
                ),
                "tokyo-night" => (
                    "#414868", "#7aa2f7", "#e0af68", "#283457", "#7dcfff", "#e0af68", "#f7768e",
                ),
                "gruvbox" => (
                    "#504945", "#83a598", "#fabd2f", "#3c3836", "#fe8019", "#fabd2f", "#fb4934",
                ),
                "dracula" => (
                    "#44475a", "#bd93f9", "#f1fa8c", "#44475a", "#8be9fd", "#ffb86c", "#ff5555",
                ),
                "nord" => (
                    "#4c566a", "#88c0d0", "#ebcb8b", "#3b4252", "#8fbcbb", "#d08770", "#bf616a",
                ),
                _ => (
                    "#555555", "#00d2ff", "#e5c07b", "#1e2d4b", "#00ffff", "#e5c07b", "#e06c75",
                ),
            };

        Self {
            border: border.into(),
            primary: primary.into(),
            secondary: secondary.into(),
            highlight_bg: hl_bg.into(),
            highlight_fg: hl_fg.into(),
            text_warn: warn.into(),
            text_crit: crit.into(),
        }
    }
}

impl Default for ResolvedTheme {
    fn default() -> Self {
        Self::from_preset("miru")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(rename = "theme", default)]
    pub section: ThemeSection,
    #[serde(skip)]
    pub theme: ResolvedTheme,
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        let path = Path::new("miru.toml");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(mut cfg) = toml::from_str::<AppConfig>(&content) {
                    cfg.rebuild_theme();
                    return cfg;
                }
            }
        }

        let mut default_cfg = AppConfig::default();
        default_cfg.rebuild_theme();
        default_cfg.save();
        default_cfg
    }

    pub fn rebuild_theme(&mut self) {
        let base = ResolvedTheme::from_preset(&self.section.preset);
        self.theme = ResolvedTheme {
            border: self.section.border.clone().unwrap_or(base.border),
            primary: self.section.primary.clone().unwrap_or(base.primary),
            secondary: self.section.secondary.clone().unwrap_or(base.secondary),
            highlight_bg: self.section.highlight_bg.clone().unwrap_or(base.highlight_bg),
            highlight_fg: self.section.highlight_fg.clone().unwrap_or(base.highlight_fg),
            text_warn: self.section.text_warn.clone().unwrap_or(base.text_warn),
            text_crit: self.section.text_crit.clone().unwrap_or(base.text_crit),
        };
    }

    pub fn preview_preset(&mut self, name: &str) {
        self.section.preset = name.to_string();
        self.rebuild_theme();
    }

    pub fn apply_and_save_preset(&mut self, name: &str) {
        self.section = ThemeSection {
            preset: name.to_string(),
            ..Default::default()
        };
        self.rebuild_theme();
        self.save();
    }

    pub fn save(&self) {
        let path = Path::new("miru.toml");
        if let Ok(serialized) = toml::to_string_pretty(self) {
            let _ = fs::write(path, serialized);
        }
    }
}

pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    Color::Cyan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff00aa"), Color::Rgb(255, 0, 170));
        assert_eq!(parse_hex_color("00d2ff"), Color::Rgb(0, 210, 255));
        assert_eq!(parse_hex_color("invalid"), Color::Cyan);
    }

    #[test]
    fn test_preset_fallback_and_override() {
        let mut cfg = AppConfig {
            section: ThemeSection {
                preset: "tokyo-night".to_string(),
                primary: Some("#123456".to_string()),
                ..Default::default()
            },
            theme: ResolvedTheme::default(),
        };

        cfg.rebuild_theme();
        // Переопределённое поле применилось
        assert_eq!(cfg.theme.primary, "#123456");
        // Остальные подтянулись из tokyo-night
        assert_eq!(cfg.theme.border, "#414868");

        // Неизвестный пресет откатывается на дефолтный miru
        let fallback = ResolvedTheme::from_preset("non-existent");
        assert_eq!(fallback.primary, "#00d2ff");
    }

    #[test]
    fn test_clean_toml_serialization() {
        let mut cfg = AppConfig::default();
        cfg.preview_preset("catppuccin");

        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        assert!(toml_str.contains("preset = \"catppuccin\""));
        assert!(!toml_str.contains("border"));
        assert!(!toml_str.contains("primary"));
    }
}