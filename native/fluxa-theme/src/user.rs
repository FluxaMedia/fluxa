use std::{
    path::Path,
    sync::{OnceLock, RwLock},
};

use serde_json::Value;

use crate::theme::{ThemeError, ThemePack, UiTokens};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserTheme {
    colors: Vec<(&'static str, String)>,
    shape: Vec<(&'static str, f64)>,
    spacing: Vec<(&'static str, f64)>,
    motion_enabled: Option<bool>,
    motion_ms: Vec<(&'static str, f64)>,
    tokens: Vec<TokenOverride>,
    hidden_rows: Vec<String>,
    row_order: Vec<String>,
    hero_actions: Option<Vec<String>>,
    detail_actions: Option<Vec<String>>,
    nav_items: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
struct TokenOverride {
    platform: Option<&'static str>,
    group: &'static str,
    key: String,
    value: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserThemeReport {
    pub applied: usize,
    pub rejected: Vec<String>,
}

const COLOR_KEYS: [&str; 20] = [
    "background",
    "backgroundElevated",
    "surface",
    "surfaceRaised",
    "navigation",
    "textPrimary",
    "textSecondary",
    "textMuted",
    "border",
    "borderStrong",
    "accent",
    "accentForeground",
    "success",
    "warning",
    "error",
    "info",
    "focus",
    "scrim",
    "buttonPrimary",
    "buttonPrimaryText",
];

const SHAPE_KEYS: [&str; 3] = ["cardRadius", "controlRadius", "dialogRadius"];
const SPACING_KEYS: [&str; 3] = ["screenPadding", "sectionGap", "controlGap"];
const MOTION_KEYS: [&str; 3] = ["fastMs", "normalMs", "slowMs"];
const COMMON_GROUPS: [&str; 5] = ["dp", "sp", "durationMs", "alpha", "number"];
const PLATFORM_GROUPS: [&str; 3] = ["dp", "sp", "player"];
const PLATFORMS: [&str; 3] = ["mobile", "desktop", "tv"];
const HERO_ACTIONS: [&str; 2] = ["play", "watchlist"];
const DETAIL_ACTIONS: [&str; 5] = ["play", "watchlist", "shuffle", "completed", "dropped"];
const NAV_ITEMS: [&str; 4] = ["home", "discover", "library", "shorts"];

static USER_THEME: OnceLock<RwLock<UserTheme>> = OnceLock::new();

fn user_theme() -> &'static RwLock<UserTheme> {
    USER_THEME.get_or_init(|| RwLock::new(UserTheme::default()))
}

pub fn set_user_theme_json(source: &str) -> UserThemeReport {
    let (theme, report) = parse_user_theme(source);
    if let Ok(mut current) = user_theme().write() {
        *current = theme;
    }
    crate::theme::invalidate_active();
    report
}

pub fn set_user_theme_from_path(path: impl AsRef<Path>) -> Result<UserThemeReport, ThemeError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|error| ThemeError::InvalidSource(format!("{}: {error}", path.display())))?;
    Ok(set_user_theme_json(&source))
}

pub fn clear_user_theme() {
    if let Ok(mut current) = user_theme().write() {
        *current = UserTheme::default();
    }
    crate::theme::invalidate_active();
}

pub fn hidden_rows() -> Vec<String> {
    user_theme()
        .read()
        .map(|user| user.hidden_rows.clone())
        .unwrap_or_default()
}

pub fn row_order() -> Vec<String> {
    user_theme()
        .read()
        .map(|user| user.row_order.clone())
        .unwrap_or_default()
}

pub fn hero_actions() -> Vec<String> {
    user_theme()
        .read()
        .ok()
        .and_then(|user| user.hero_actions.clone())
        .unwrap_or_else(|| HERO_ACTIONS.map(str::to_owned).to_vec())
}

pub fn detail_actions() -> Option<Vec<String>> {
    user_theme()
        .read()
        .ok()
        .and_then(|user| user.detail_actions.clone())
}

pub fn nav_items() -> Option<Vec<String>> {
    user_theme()
        .read()
        .ok()
        .and_then(|user| user.nav_items.clone())
}

pub(crate) fn apply_user_theme(theme: &mut ThemePack) {
    if let Ok(user) = user_theme().read() {
        user.apply_to_theme(theme);
    }
}

pub(crate) fn apply_user_tokens(tokens: &mut UiTokens) {
    if let Ok(user) = user_theme().read() {
        user.apply_to_tokens(tokens);
    }
}

impl UserTheme {
    fn apply_to_theme(&self, theme: &mut ThemePack) {
        for (name, value) in &self.colors {
            let slot = match *name {
                "buttonPrimary" => {
                    theme.colors.button_primary = Some(value.clone());
                    continue;
                }
                "buttonPrimaryText" => {
                    theme.colors.button_primary_text = Some(value.clone());
                    continue;
                }
                "background" => &mut theme.colors.background,
                "backgroundElevated" => &mut theme.colors.background_elevated,
                "surface" => &mut theme.colors.surface,
                "surfaceRaised" => &mut theme.colors.surface_raised,
                "navigation" => &mut theme.colors.navigation,
                "textPrimary" => &mut theme.colors.text_primary,
                "textSecondary" => &mut theme.colors.text_secondary,
                "textMuted" => &mut theme.colors.text_muted,
                "border" => &mut theme.colors.border,
                "borderStrong" => &mut theme.colors.border_strong,
                "accent" => &mut theme.colors.accent,
                "accentForeground" => &mut theme.colors.accent_foreground,
                "success" => &mut theme.colors.success,
                "warning" => &mut theme.colors.warning,
                "error" => &mut theme.colors.error,
                "info" => &mut theme.colors.info,
                "focus" => &mut theme.colors.focus,
                _ => &mut theme.colors.scrim,
            };
            slot.clone_from(value);
        }
        for (name, value) in &self.shape {
            let value = *value as f32;
            match *name {
                "cardRadius" => theme.shape.card_radius = value,
                "controlRadius" => theme.shape.control_radius = value,
                _ => theme.shape.dialog_radius = value,
            }
        }
        for (name, value) in &self.spacing {
            let value = *value as f32;
            match *name {
                "screenPadding" => theme.spacing.screen_padding = value,
                "sectionGap" => theme.spacing.section_gap = value,
                _ => theme.spacing.control_gap = value,
            }
        }
        if let Some(enabled) = self.motion_enabled {
            theme.motion.enabled = enabled;
        }
        for (name, value) in &self.motion_ms {
            let value = *value as u64;
            match *name {
                "fastMs" => theme.motion.fast_ms = value,
                "normalMs" => theme.motion.normal_ms = value,
                _ => theme.motion.slow_ms = value,
            }
        }
    }

    fn apply_to_tokens(&self, tokens: &mut UiTokens) {
        for entry in &self.tokens {
            let scope = match entry.platform {
                Some(platform) => tokens
                    .layout
                    .get_mut("platforms")
                    .and_then(|platforms| platforms.get_mut(platform)),
                None => tokens.layout.get_mut("common"),
            };
            let slot = scope
                .and_then(|scope| scope.get_mut(entry.group))
                .and_then(|group| group.get_mut(entry.key.as_str()));
            if let Some(slot) = slot
                && slot.is_number()
            {
                *slot = Value::from(entry.value);
            }
        }
    }
}

pub fn parse_user_theme(source: &str) -> (UserTheme, UserThemeReport) {
    let mut theme = UserTheme::default();
    let mut report = UserThemeReport::default();
    let root = match serde_json::from_str::<Value>(source) {
        Ok(Value::Object(root)) => root,
        Ok(_) => {
            report
                .rejected
                .push("the theme must be a JSON object".to_owned());
            return (theme, report);
        }
        Err(error) => {
            report.rejected.push(format!("invalid JSON: {error}"));
            return (theme, report);
        }
    };
    for (section, body) in &root {
        match section.as_str() {
            "schemaVersion" | "name" => {}
            "colors" => parse_colors(&mut theme, body, &mut report),
            "shape" => {
                theme.shape = numeric_section("shape", body, &SHAPE_KEYS, 0.0, 64.0, &mut report);
            }
            "spacing" => {
                theme.spacing =
                    numeric_section("spacing", body, &SPACING_KEYS, 0.0, 96.0, &mut report);
            }
            "motion" => parse_motion(&mut theme, body, &mut report),
            "tokens" => parse_tokens(&mut theme, body, &mut report),
            "home" => parse_home(&mut theme, body, &mut report),
            other => report.rejected.push(format!("{other}: unknown section")),
        }
    }
    (theme, report)
}

fn entries<'a>(
    section: &str,
    body: &'a Value,
    report: &mut UserThemeReport,
) -> Vec<(&'a String, &'a Value)> {
    match body.as_object() {
        Some(map) => map.iter().collect(),
        None => {
            report
                .rejected
                .push(format!("{section}: expected an object"));
            Vec::new()
        }
    }
}

fn number(value: &Value, min: f64, max: f64) -> Option<f64> {
    value
        .as_f64()
        .filter(|number| number.is_finite() && (min..=max).contains(number))
}

fn parse_colors(theme: &mut UserTheme, body: &Value, report: &mut UserThemeReport) {
    for (key, value) in entries("colors", body, report) {
        let Some(name) = COLOR_KEYS.iter().find(|known| **known == key) else {
            report.rejected.push(format!("colors.{key}: unknown color"));
            continue;
        };
        match value.as_str().and_then(parse_color) {
            Some(hex) => {
                theme.colors.retain(|(existing, _)| existing != name);
                theme.colors.push((name, hex));
                report.applied += 1;
            }
            None => report
                .rejected
                .push(format!("colors.{key}: invalid color `{value}`")),
        }
    }
}

fn numeric_section(
    section: &str,
    body: &Value,
    keys: &[&'static str],
    min: f64,
    max: f64,
    report: &mut UserThemeReport,
) -> Vec<(&'static str, f64)> {
    let mut parsed: Vec<(&'static str, f64)> = Vec::new();
    for (key, value) in entries(section, body, report) {
        let Some(name) = keys.iter().find(|known| **known == key) else {
            report
                .rejected
                .push(format!("{section}.{key}: unknown key"));
            continue;
        };
        match number(value, min, max) {
            Some(number) => {
                parsed.retain(|(existing, _)| existing != name);
                parsed.push((name, number));
                report.applied += 1;
            }
            None => report.rejected.push(format!(
                "{section}.{key}: expected a number between {min} and {max}"
            )),
        }
    }
    parsed
}

fn parse_motion(theme: &mut UserTheme, body: &Value, report: &mut UserThemeReport) {
    let mut durations = serde_json::Map::new();
    for (key, value) in entries("motion", body, report) {
        if key == "enabled" {
            match value.as_bool() {
                Some(enabled) => {
                    theme.motion_enabled = Some(enabled);
                    report.applied += 1;
                }
                None => report
                    .rejected
                    .push("motion.enabled: expected true or false".to_owned()),
            }
        } else {
            durations.insert(key.clone(), value.clone());
        }
    }
    theme.motion_ms = numeric_section(
        "motion",
        &Value::Object(durations),
        &MOTION_KEYS,
        0.0,
        2000.0,
        report,
    );
}

fn parse_tokens(theme: &mut UserTheme, body: &Value, report: &mut UserThemeReport) {
    for (key, value) in entries("tokens", body, report) {
        if key == "platforms" {
            for (platform, groups) in entries("tokens.platforms", value, report) {
                let Some(platform) = PLATFORMS.iter().find(|known| **known == platform) else {
                    report
                        .rejected
                        .push(format!("tokens.platforms.{platform}: unknown platform"));
                    continue;
                };
                token_groups(theme, Some(platform), &PLATFORM_GROUPS, groups, report);
            }
        }
    }
    let common: serde_json::Map<String, Value> = body
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| key.as_str() != "platforms")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    token_groups(theme, None, &COMMON_GROUPS, &Value::Object(common), report);
}

fn token_groups(
    theme: &mut UserTheme,
    platform: Option<&'static str>,
    allowed: &[&'static str],
    body: &Value,
    report: &mut UserThemeReport,
) {
    let base = crate::theme::shared_ui_tokens().ok();
    for (group, keys) in entries("tokens", body, report) {
        let Some(group_name) = allowed.iter().find(|known| **known == group) else {
            report
                .rejected
                .push(format!("tokens.{group}: unknown group"));
            continue;
        };
        for (key, value) in entries(group, keys, report) {
            let scope = base.and_then(|tokens| match platform {
                Some(platform) => tokens
                    .layout
                    .get("platforms")
                    .and_then(|platforms| platforms.get(platform)),
                None => tokens.layout.get("common"),
            });
            let exists = scope
                .and_then(|scope| scope.get(group_name))
                .and_then(|group| group.get(key.as_str()))
                .is_some_and(Value::is_number);
            if base.is_some() && !exists {
                report
                    .rejected
                    .push(format!("tokens.{group}.{key}: unknown token"));
                continue;
            }
            let (min, max) = token_range(group_name, key);
            match number(value, min, max) {
                Some(number) => {
                    theme.tokens.retain(|existing| {
                        !(existing.platform == platform
                            && existing.group == *group_name
                            && existing.key == *key)
                    });
                    theme.tokens.push(TokenOverride {
                        platform,
                        group: group_name,
                        key: key.clone(),
                        value: number,
                    });
                    report.applied += 1;
                }
                None => report.rejected.push(format!(
                    "tokens.{group}.{key}: expected a number between {min} and {max}"
                )),
            }
        }
    }
}

fn token_range(group: &str, key: &str) -> (f64, f64) {
    match (group, key) {
        ("alpha", _) => (0.0, 1.0),
        ("sp", _) => (1.0, 300.0),
        ("durationMs", _) => (0.0, 5000.0),
        ("number", "cardFocusedScale") => (1.0, 1.3),
        ("number", _) => (0.0, 100.0),
        _ => (0.0, 4000.0),
    }
}

fn parse_home(theme: &mut UserTheme, body: &Value, report: &mut UserThemeReport) {
    for (key, value) in entries("home", body, report) {
        match key.as_str() {
            "hideRows" | "rowOrder" => {
                let Some(list) = string_list(key, value, None, report) else {
                    continue;
                };
                if key == "hideRows" {
                    theme.hidden_rows = list;
                } else {
                    theme.row_order = list;
                }
                report.applied += 1;
            }
            "heroActions" | "detailActions" | "navItems" => {
                let known: &[&str] = match key.as_str() {
                    "heroActions" => &HERO_ACTIONS,
                    "navItems" => &NAV_ITEMS,
                    _ => &DETAIL_ACTIONS,
                };
                let Some(list) = string_list(key, value, Some(known), report) else {
                    continue;
                };
                match key.as_str() {
                    "heroActions" => theme.hero_actions = Some(list),
                    "navItems" => theme.nav_items = Some(list),
                    _ => theme.detail_actions = Some(list),
                }
                report.applied += 1;
            }
            _ => report.rejected.push(format!("home.{key}: unknown key")),
        }
    }
}

fn string_list(
    key: &str,
    value: &Value,
    known: Option<&[&str]>,
    report: &mut UserThemeReport,
) -> Option<Vec<String>> {
    let Some(items) = value.as_array() else {
        report
            .rejected
            .push(format!("home.{key}: expected an array of strings"));
        return None;
    };
    let mut list: Vec<String> = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else {
            report
                .rejected
                .push(format!("home.{key}: expected an array of strings"));
            return None;
        };
        let entry = text.trim().to_lowercase();
        if entry.is_empty() || list.contains(&entry) {
            continue;
        }
        if known.is_some_and(|known| !known.contains(&entry.as_str())) {
            report
                .rejected
                .push(format!("home.{key}: unknown entry `{entry}`"));
            continue;
        }
        list.push(entry);
    }
    Some(list)
}

fn parse_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return expand_hex(hex);
    }
    let lower = value.to_ascii_lowercase();
    let inner = lower
        .strip_prefix("rgba(")
        .or_else(|| lower.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let parts: Vec<&str> = inner
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let channel = |text: &str| {
        text.parse::<f32>()
            .ok()
            .map(|v| v.clamp(0.0, 255.0).round() as u8)
    };
    let alpha = match parts.get(3) {
        Some(text) => match text.strip_suffix('%') {
            Some(percent) => percent.parse::<f32>().ok()? / 100.0,
            None => text.parse::<f32>().ok()?,
        },
        None => 1.0,
    };
    Some(format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        channel(parts[0])?,
        channel(parts[1])?,
        channel(parts[2])?,
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
    ))
}

fn expand_hex(hex: &str) -> Option<String> {
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let upper = hex.to_ascii_uppercase();
    match upper.len() {
        3 | 4 => {
            let doubled: String = upper.chars().flat_map(|c| [c, c]).collect();
            expand_hex(&doubled)
        }
        6 => Some(format!("#{upper}FF")),
        8 => Some(format!("#{upper}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{shared_ui_tokens, theme};

    #[test]
    fn row_lists_are_lowercased_and_deduplicated() {
        let (user, _) = parse_user_theme(
            r#"{"home": {"hideRows": ["Top 10", " Popular ", "top 10"], "rowOrder": ["a", "b"]}}"#,
        );
        assert_eq!(user.hidden_rows, vec!["top 10", "popular"]);
        assert_eq!(user.row_order, vec!["a", "b"]);
    }

    #[test]
    fn hero_actions_keep_order_and_report_unknown_names() {
        let (user, report) = parse_user_theme(
            r#"{"home": {"heroActions": ["watchlist", "share", "play", "play"]}}"#,
        );
        assert_eq!(
            user.hero_actions,
            Some(vec!["watchlist".to_owned(), "play".to_owned()])
        );
        assert_eq!(report.rejected.len(), 1);
    }

    #[test]
    fn bad_values_are_reported_and_good_ones_still_apply() {
        let json = r##"{
            "colors": {"accent": "#e50914", "surface": "nope", "bogus": "#fff"},
            "shape": {"cardRadius": 6, "controlRadius": 500},
            "extra": {}
        }"##;
        let (user, report) = parse_user_theme(json);
        assert_eq!(report.applied, 2);
        assert_eq!(report.rejected.len(), 4);
        assert_eq!(user.colors, vec![("accent", "#E50914FF".to_owned())]);
        assert_eq!(user.shape, vec![("cardRadius", 6.0)]);
    }

    #[test]
    fn invalid_json_is_rejected_without_panicking() {
        let (user, report) = parse_user_theme("{ nope");
        assert_eq!(user, UserTheme::default());
        assert_eq!(report.rejected.len(), 1);
    }

    #[test]
    fn theme_overrides_reach_the_theme_pack() {
        let (user, _) = parse_user_theme(
            r##"{"colors": {"accent": "#112233"}, "shape": {"cardRadius": 4},
                "spacing": {"sectionGap": 30}, "motion": {"enabled": false, "fastMs": 50}}"##,
        );
        let mut pack = theme("fluxa-dark").expect("theme").clone();
        user.apply_to_theme(&mut pack);
        assert_eq!(pack.colors.accent, "#112233FF");
        assert_eq!(pack.shape.card_radius, 4.0);
        assert_eq!(pack.spacing.section_gap, 30.0);
        assert!(!pack.motion.enabled);
        assert_eq!(pack.motion.fast_ms, 50);
        assert_eq!(pack.motion.slow_ms, 300);
    }

    #[test]
    fn token_overrides_respect_scope_and_limits() {
        let json = r#"{"tokens": {
            "number": {"cardFocusedScale": 1.2, "posterHeightRatio": 1000},
            "alpha": {"hairline": 0.1, "notAToken": 0.2},
            "platforms": {"tv": {"dp": {"buttonHeight": 72}}, "watch": {}}
        }}"#;
        let (user, report) = parse_user_theme(json);
        assert_eq!(report.applied, 3);
        assert_eq!(report.rejected.len(), 3);

        let mut tokens = shared_ui_tokens().expect("tokens").clone();
        user.apply_to_tokens(&mut tokens);
        let layout = &tokens.layout;
        assert_eq!(layout["common"]["number"]["cardFocusedScale"], 1.2);
        assert_eq!(layout["common"]["alpha"]["hairline"], 0.1);
        assert_eq!(layout["platforms"]["tv"]["dp"]["buttonHeight"], 72.0);
        assert_eq!(
            layout["platforms"]["desktop"]["dp"]["buttonHeight"],
            shared_ui_tokens().expect("tokens").layout["platforms"]["desktop"]["dp"]["buttonHeight"]
        );
    }

    #[test]
    fn button_colors_are_optional_theme_fields() {
        let base = theme("fluxa-dark").expect("theme").clone();
        assert_eq!(base.colors.button_primary, None);
        let (user, report) = parse_user_theme(
            r##"{"colors": {"buttonPrimary": "#E8A33D", "buttonPrimaryText": "#000"},
                "tokens": {"number": {"pillRadiusRatio": 0.2}}}"##,
        );
        assert!(report.rejected.is_empty());
        let mut pack = base;
        user.apply_to_theme(&mut pack);
        assert_eq!(pack.colors.button_primary.as_deref(), Some("#E8A33DFF"));
        assert!(pack.color("buttonPrimaryText").is_ok());
    }

    #[test]
    fn short_hex_and_rgba_normalise_to_eight_digits() {
        assert_eq!(parse_color("#fff").as_deref(), Some("#FFFFFFFF"));
        assert_eq!(parse_color("#0008").as_deref(), Some("#00000088"));
        assert_eq!(
            parse_color("rgba(255, 0, 0, 0.5)").as_deref(),
            Some("#FF000080")
        );
        assert_eq!(
            parse_color("rgb(10 20 30 / 50%)").as_deref(),
            Some("#0A141E80")
        );
        assert_eq!(parse_color("#12345"), None);
    }
}
