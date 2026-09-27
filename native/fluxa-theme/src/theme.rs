//! Shared Fluxa theme/token contract.
//!
//! `shared/contracts/ui-tokens.json` is the source of truth for Web, Compose,
//! and native Rust. The renderer deliberately consumes the same document
//! instead of keeping a second set of Rust-only design constants.

use std::{
    fmt,
    path::Path,
    sync::{OnceLock, RwLock},
};

use serde::Deserialize;

use crate::Color;

const SHARED_TOKENS_JSON: &str = include_str!("../../../shared/contracts/ui-tokens.json");

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UiTokens {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub colors: BaseColors,
    pub shape: BaseShape,
    pub spacing: BaseSpacing,
    pub auth: serde_json::Value,
    pub typography: BaseTypography,
    pub layout: serde_json::Value,
    pub themes: Vec<ThemePack>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BaseColors {
    pub background: String,
    pub surface: String,
    #[serde(rename = "surfaceRaised")]
    pub surface_raised: String,
    #[serde(rename = "textPrimary")]
    pub text_primary: String,
    #[serde(rename = "textSecondary")]
    pub text_secondary: String,
    #[serde(rename = "textMuted")]
    pub text_muted: String,
    pub border: String,
    #[serde(rename = "borderStrong")]
    pub border_strong: String,
    pub focus: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BaseShape {
    #[serde(rename = "authCard")]
    pub auth_card: f32,
    pub control: f32,
    pub segmented: f32,
    pub tab: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BaseSpacing {
    pub screen: f32,
    #[serde(rename = "authSection")]
    pub auth_section: f32,
    #[serde(rename = "authCardPadding")]
    pub auth_card_padding: f32,
    pub control: f32,
    pub divider: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BaseTypography {
    #[serde(rename = "authTitle")]
    pub auth_title: f32,
    #[serde(rename = "authSubtitle")]
    pub auth_subtitle: f32,
    #[serde(rename = "authControl")]
    pub auth_control: f32,
    #[serde(rename = "authCaption")]
    pub auth_caption: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemePack {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub id: String,
    #[serde(rename = "nameKey")]
    pub name_key: String,
    #[serde(default)]
    pub name: Option<String>,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub shape: ThemeShape,
    pub spacing: ThemeSpacing,
    pub motion: ThemeMotion,
    pub layouts: ThemeLayouts,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeColors {
    pub background: String,
    #[serde(rename = "backgroundElevated")]
    pub background_elevated: String,
    pub surface: String,
    #[serde(rename = "surfaceRaised")]
    pub surface_raised: String,
    pub navigation: String,
    #[serde(rename = "textPrimary")]
    pub text_primary: String,
    #[serde(rename = "textSecondary")]
    pub text_secondary: String,
    #[serde(rename = "textMuted")]
    pub text_muted: String,
    pub border: String,
    #[serde(rename = "borderStrong")]
    pub border_strong: String,
    pub accent: String,
    #[serde(rename = "accentForeground")]
    pub accent_foreground: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub focus: String,
    pub scrim: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeTypography {
    #[serde(rename = "displayFont")]
    pub display_font: String,
    #[serde(rename = "bodyFont")]
    pub body_font: String,
    #[serde(rename = "titleWeight")]
    pub title_weight: u16,
    #[serde(rename = "bodyWeight")]
    pub body_weight: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeShape {
    #[serde(rename = "cardRadius")]
    pub card_radius: f32,
    #[serde(rename = "controlRadius")]
    pub control_radius: f32,
    #[serde(rename = "dialogRadius")]
    pub dialog_radius: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeSpacing {
    #[serde(rename = "screenPadding")]
    pub screen_padding: f32,
    #[serde(rename = "sectionGap")]
    pub section_gap: f32,
    #[serde(rename = "controlGap")]
    pub control_gap: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeMotion {
    pub enabled: bool,
    #[serde(rename = "fastMs")]
    pub fast_ms: u64,
    #[serde(rename = "normalMs")]
    pub normal_ms: u64,
    #[serde(rename = "slowMs")]
    pub slow_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ThemeLayouts {
    pub home: String,
    pub detail: String,
    pub library: String,
    pub navigation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    InvalidSource(String),
    MissingTheme(String),
    InvalidColor { token: String, value: String },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource(error) => write!(formatter, "invalid shared UI tokens: {error}"),
            Self::MissingTheme(id) => write!(formatter, "shared theme not found: {id}"),
            Self::InvalidColor { token, value } => {
                write!(formatter, "invalid color token {token}: {value}")
            }
        }
    }
}

impl std::error::Error for ThemeError {}

static TOKENS: OnceLock<Result<UiTokens, ThemeError>> = OnceLock::new();
static LIVE_TOKENS: OnceLock<RwLock<Option<UiTokens>>> = OnceLock::new();

fn live_tokens() -> &'static RwLock<Option<UiTokens>> {
    LIVE_TOKENS.get_or_init(|| RwLock::new(None))
}

/// Parse and cache the repository's shared token document.
pub fn shared_ui_tokens() -> Result<&'static UiTokens, ThemeError> {
    TOKENS
        .get_or_init(|| {
            serde_json::from_str(SHARED_TOKENS_JSON)
                .map_err(|error| ThemeError::InvalidSource(error.to_string()))
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn theme(id: &str) -> Result<&'static ThemePack, ThemeError> {
    shared_ui_tokens()?
        .themes
        .iter()
        .find(|theme| theme.id == id)
        .ok_or_else(|| ThemeError::MissingTheme(id.to_owned()))
}

/// Return the active token document, including an optional development-time
/// override loaded from disk. Production callers continue to use the
/// compile-time contract; desktop development can replace only the token
/// document without rebuilding or reopening the window.
pub fn active_shared_ui_tokens() -> Result<UiTokens, ThemeError> {
    if let Some(tokens) = live_tokens().read().ok().and_then(|tokens| tokens.clone()) {
        return Ok(tokens);
    }
    shared_ui_tokens().cloned()
}

pub fn active_theme(id: &str) -> Result<ThemePack, ThemeError> {
    active_shared_ui_tokens()?
        .themes
        .into_iter()
        .find(|theme| theme.id == id)
        .ok_or_else(|| ThemeError::MissingTheme(id.to_owned()))
}

/// Replace the development-time token override with a JSON file from disk.
/// Invalid files are rejected without disturbing the last valid theme.
pub fn reload_shared_ui_tokens_from_path(path: impl AsRef<Path>) -> Result<(), ThemeError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|error| ThemeError::InvalidSource(format!("{}: {error}", path.display())))?;
    let tokens = serde_json::from_str::<UiTokens>(&source)
        .map_err(|error| ThemeError::InvalidSource(error.to_string()))?;
    *live_tokens()
        .write()
        .map_err(|_| ThemeError::InvalidSource("token reload lock poisoned".to_owned()))? =
        Some(tokens);
    Ok(())
}

impl ThemePack {
    pub fn color(&self, token: &str) -> Result<Color, ThemeError> {
        let value = match token {
            "background" => &self.colors.background,
            "backgroundElevated" => &self.colors.background_elevated,
            "surface" => &self.colors.surface,
            "surfaceRaised" => &self.colors.surface_raised,
            "navigation" => &self.colors.navigation,
            "textPrimary" => &self.colors.text_primary,
            "textSecondary" => &self.colors.text_secondary,
            "textMuted" => &self.colors.text_muted,
            "border" => &self.colors.border,
            "borderStrong" => &self.colors.border_strong,
            "accent" => &self.colors.accent,
            "accentForeground" => &self.colors.accent_foreground,
            "success" => &self.colors.success,
            "warning" => &self.colors.warning,
            "error" => &self.colors.error,
            "info" => &self.colors.info,
            "focus" => &self.colors.focus,
            "scrim" => &self.colors.scrim,
            _ => {
                return Err(ThemeError::InvalidColor {
                    token: token.to_owned(),
                    value: "unknown token".to_owned(),
                });
            }
        };
        parse_hex_color(token, value)
    }
}

fn parse_hex_color(token: &str, value: &str) -> Result<Color, ThemeError> {
    let raw = value.strip_prefix('#').unwrap_or(value);
    let parse = |text: &str| u8::from_str_radix(text, 16).ok();
    let (r, g, b, a) = match raw.len() {
        6 => (
            parse(&raw[0..2]),
            parse(&raw[2..4]),
            parse(&raw[4..6]),
            Some(255),
        ),
        8 => (
            parse(&raw[0..2]),
            parse(&raw[2..4]),
            parse(&raw[4..6]),
            parse(&raw[6..8]),
        ),
        _ => (None, None, None, None),
    };
    match (r, g, b, a) {
        (Some(r), Some(g), Some(b), Some(a)) => Ok(Color::rgba(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        )),
        _ => Err(ThemeError::InvalidColor {
            token: token.to_owned(),
            value: value.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_source_contains_all_builtin_themes() {
        let tokens = shared_ui_tokens().expect("shared token source must parse");
        assert_eq!(tokens.schema_version, 3);
        assert_eq!(tokens.themes.len(), 3);
        assert_eq!(
            theme("midnight-blue").expect("theme").layouts.home,
            "shelves"
        );
    }

    #[test]
    fn hex_tokens_convert_to_unit_rgb() {
        let color = parse_hex_color("accent", "#E85D3F").expect("color");
        assert_eq!(color, Color::rgb(232.0 / 255.0, 93.0 / 255.0, 63.0 / 255.0));
        assert!(theme("fluxa-dark").expect("theme").color("accent").is_ok());
    }
}
