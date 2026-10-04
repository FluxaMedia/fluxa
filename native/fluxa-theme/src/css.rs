use std::{
    path::Path,
    sync::{OnceLock, RwLock},
};

use crate::theme::{ThemeError, ThemePack};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserTheme {
    colors: Vec<(&'static str, String)>,
    radii: Vec<(&'static str, f32)>,
    hidden_rows: Vec<String>,
    row_order: Vec<String>,
    hero_actions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssReport {
    pub applied: usize,
    pub rejected: Vec<String>,
}

const COLOR_PROPERTIES: [&str; 18] = [
    "background",
    "background-elevated",
    "surface",
    "surface-raised",
    "navigation",
    "text-primary",
    "text-secondary",
    "text-muted",
    "border",
    "border-strong",
    "accent",
    "accent-foreground",
    "success",
    "warning",
    "error",
    "info",
    "focus",
    "scrim",
];

const RADIUS_PROPERTIES: [&str; 2] = ["card-radius", "dialog-radius"];

static USER_THEME: OnceLock<RwLock<UserTheme>> = OnceLock::new();

fn user_theme() -> &'static RwLock<UserTheme> {
    USER_THEME.get_or_init(|| RwLock::new(UserTheme::default()))
}

pub fn set_user_theme_css(source: &str) -> CssReport {
    let (theme, report) = parse_user_theme(source);
    if let Ok(mut current) = user_theme().write() {
        *current = theme;
    }
    report
}

pub fn set_user_theme_from_path(path: impl AsRef<Path>) -> Result<CssReport, ThemeError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|error| ThemeError::InvalidSource(format!("{}: {error}", path.display())))?;
    Ok(set_user_theme_css(&source))
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
        .unwrap_or_else(|| vec!["play".to_owned(), "watchlist".to_owned()])
}

pub fn clear_user_theme() {
    if let Ok(mut current) = user_theme().write() {
        *current = UserTheme::default();
    }
}

pub(crate) fn apply_user_theme(theme: &mut ThemePack) {
    let Ok(user) = user_theme().read() else {
        return;
    };
    for (name, value) in &user.colors {
        let slot = match *name {
            "background" => &mut theme.colors.background,
            "background-elevated" => &mut theme.colors.background_elevated,
            "surface" => &mut theme.colors.surface,
            "surface-raised" => &mut theme.colors.surface_raised,
            "navigation" => &mut theme.colors.navigation,
            "text-primary" => &mut theme.colors.text_primary,
            "text-secondary" => &mut theme.colors.text_secondary,
            "text-muted" => &mut theme.colors.text_muted,
            "border" => &mut theme.colors.border,
            "border-strong" => &mut theme.colors.border_strong,
            "accent" => &mut theme.colors.accent,
            "accent-foreground" => &mut theme.colors.accent_foreground,
            "success" => &mut theme.colors.success,
            "warning" => &mut theme.colors.warning,
            "error" => &mut theme.colors.error,
            "info" => &mut theme.colors.info,
            "focus" => &mut theme.colors.focus,
            _ => &mut theme.colors.scrim,
        };
        slot.clone_from(value);
    }
    for (name, value) in &user.radii {
        match *name {
            "card-radius" => theme.shape.card_radius = *value,
            _ => theme.shape.dialog_radius = *value,
        }
    }
}

pub fn parse_user_theme(source: &str) -> (UserTheme, CssReport) {
    let mut theme = UserTheme::default();
    let mut report = CssReport::default();
    for (name, value) in root_declarations(&strip_comments(source)) {
        let Some(name) = name.strip_prefix("--") else {
            report
                .rejected
                .push(format!("{name}: only custom properties are supported"));
            continue;
        };
        if let Some(property) = COLOR_PROPERTIES.iter().find(|known| **known == name) {
            match parse_color(&value) {
                Some(hex) => {
                    theme.colors.retain(|(existing, _)| existing != property);
                    theme.colors.push((property, hex));
                    report.applied += 1;
                }
                None => report
                    .rejected
                    .push(format!("--{name}: invalid color `{value}`")),
            }
        } else if let Some(property) = RADIUS_PROPERTIES.iter().find(|known| **known == name) {
            match parse_length(&value) {
                Some(radius) => {
                    theme.radii.retain(|(existing, _)| existing != property);
                    theme.radii.push((property, radius));
                    report.applied += 1;
                }
                None => report
                    .rejected
                    .push(format!("--{name}: invalid length `{value}`")),
            }
        } else if name == "hide-rows" || name == "row-order" {
            let list = value
                .split(',')
                .map(|entry| entry.trim().trim_matches('"').to_lowercase())
                .filter(|entry| !entry.is_empty())
                .collect();
            if name == "hide-rows" {
                theme.hidden_rows = list;
            } else {
                theme.row_order = list;
            }
            report.applied += 1;
        } else if name == "hero-actions" {
            let mut list: Vec<String> = Vec::new();
            for entry in value.split(',').map(|entry| entry.trim().to_lowercase()) {
                if matches!(entry.as_str(), "play" | "watchlist") && !list.contains(&entry) {
                    list.push(entry);
                }
            }
            theme.hero_actions = Some(list);
            report.applied += 1;
        } else {
            report.rejected.push(format!("--{name}: unknown property"));
        }
    }
    (theme, report)
}

fn strip_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        output.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return output,
        }
    }
    output.push_str(rest);
    output
}

fn root_declarations(source: &str) -> Vec<(String, String)> {
    let mut declarations = Vec::new();
    let mut rest = source;
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim();
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let body = &rest[open + 1..open + close];
        rest = &rest[open + close + 1..];
        if selector != ":root" {
            continue;
        }
        for declaration in body.split(';') {
            if let Some((name, value)) = declaration.split_once(':') {
                let (name, value) = (name.trim(), value.trim());
                if !name.is_empty() && !value.is_empty() {
                    declarations.push((name.to_owned(), value.to_owned()));
                }
            }
        }
    }
    declarations
}

fn parse_length(value: &str) -> Option<f32> {
    let number = value.strip_suffix("px").unwrap_or(value).trim();
    let radius = number.parse::<f32>().ok()?;
    (radius.is_finite() && (0.0..=999.0).contains(&radius)).then_some(radius)
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

    #[test]
    fn row_lists_are_lowercased_and_split_on_commas() {
        let (user, _) = parse_user_theme(":root { --hide-rows: Top 10, \"Popular\"; --row-order: a,b }");
        assert_eq!(user.hidden_rows, vec!["top 10", "popular"]);
        assert_eq!(user.row_order, vec!["a", "b"]);
    }

    #[test]
    fn hero_actions_keep_order_and_drop_unknown_names() {
        let (user, _) = parse_user_theme(":root { --hero-actions: watchlist, share, play, play }");
        assert_eq!(user.hero_actions, Some(vec!["watchlist".to_owned(), "play".to_owned()]));
    }

    #[test]
    fn root_block_overrides_apply_and_bad_values_are_reported() {
        let css = ":root { --accent: #e50914; --card-radius: 6px; --surface: nope; --bogus: 1; }\n.card { color: red }";
        let (user, report) = parse_user_theme(css);
        assert_eq!(report.applied, 2);
        assert_eq!(report.rejected.len(), 2);
        assert_eq!(user.colors, vec![("accent", "#E50914FF".to_owned())]);
        assert_eq!(user.radii, vec![("card-radius", 6.0)]);
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

    #[test]
    fn comments_are_ignored() {
        let (user, _) = parse_user_theme("/* x */ :root { /* y */ --focus: #123456; }");
        assert_eq!(user.colors, vec![("focus", "#123456FF".to_owned())]);
    }
}
