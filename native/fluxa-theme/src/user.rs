use std::{
    hash::{Hash, Hasher},
    path::{Component, Path},
    sync::{Arc, OnceLock, RwLock},
};

use serde_json::Value;

use crate::theme::ThemeError;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserTheme {
    colors: Vec<(&'static str, String)>,
    shape: Vec<(&'static str, f64)>,
    spacing: Vec<(&'static str, f64)>,
    motion_enabled: Option<bool>,
    motion_ms: Vec<(&'static str, f64)>,
    vars: Vec<(String, f64)>,
    hidden_rows: Vec<String>,
    row_order: Vec<String>,
    hero_actions: Option<Vec<String>>,
    detail_actions: Option<Vec<String>>,
    nav_items: Option<Vec<String>>,
    name: Option<String>,
    base: Option<String>,
    fonts: Vec<(&'static str, Arc<Vec<u8>>)>,
    background: Option<Arc<Vec<u8>>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserThemeReport {
    pub applied: usize,
    pub rejected: Vec<String>,
}

const COLOR_KEYS: [&str; 24] = [
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
    "cardBackground",
    "inputBackground",
    "inputBorder",
    "inputBorderFocus",
];

const SHAPE_KEYS: [&str; 3] = ["cardRadius", "controlRadius", "dialogRadius"];
const SPACING_KEYS: [&str; 3] = ["screenPadding", "sectionGap", "controlGap"];
const MOTION_KEYS: [&str; 3] = ["fastMs", "normalMs", "slowMs"];
const HERO_ACTIONS: [&str; 2] = ["play", "watchlist"];
const DETAIL_ACTIONS: [&str; 5] = ["play", "watchlist", "shuffle", "completed", "dropped"];
const NAV_ITEMS: [&str; 3] = ["home", "discover", "library"];

static USER_THEME: OnceLock<RwLock<UserTheme>> = OnceLock::new();
static FILE_THEME: RwLock<Option<UserTheme>> = RwLock::new(None);
static PACKS: RwLock<Vec<Pack>> = RwLock::new(Vec::new());

const PACK_LIMIT: usize = 32;
const FONT_WEIGHTS: [&str; 4] = ["regular", "medium", "semibold", "bold"];
const FONT_LIMIT: u64 = 16 * 1024 * 1024;
const IMAGE_LIMIT: u64 = 24 * 1024 * 1024;

struct Pack {
    id: String,
    name: String,
    base: String,
    theme: UserTheme,
}

fn user_theme() -> &'static RwLock<UserTheme> {
    USER_THEME.get_or_init(|| RwLock::new(UserTheme::default()))
}

pub fn set_user_theme_json(source: &str) -> UserThemeReport {
    let (theme, report) = parse_user_theme(source);
    if let Ok(mut file) = FILE_THEME.write() {
        *file = Some(theme);
    }
    activate(&crate::theme::selected_theme_key());
    crate::theme::invalidate_active();
    report
}

pub(crate) fn activate(key: &str) {
    let theme = pack_theme(key)
        .or_else(|| FILE_THEME.read().ok().and_then(|file| file.clone()))
        .unwrap_or_default();
    if let Ok(mut current) = user_theme().write() {
        *current = theme;
    }
}

fn pack_theme(key: &str) -> Option<UserTheme> {
    let id = key.strip_prefix("pack:")?;
    let packs = PACKS.read().ok()?;
    packs
        .iter()
        .find(|pack| pack.id == id)
        .map(|pack| pack.theme.clone())
}

pub(crate) fn pack_base(key: &str) -> Option<String> {
    let id = key.strip_prefix("pack:")?;
    let packs = PACKS.read().ok()?;
    packs
        .iter()
        .find(|pack| pack.id == id)
        .map(|pack| pack.base.clone())
}

pub(crate) fn pack_keys() -> Vec<String> {
    PACKS
        .read()
        .map(|packs| {
            packs
                .iter()
                .map(|pack| format!("pack:{}", pack.id))
                .collect()
        })
        .unwrap_or_default()
}

pub fn pack_name(key: &str) -> Option<String> {
    let id = key.strip_prefix("pack:")?;
    let packs = PACKS.read().ok()?;
    packs
        .iter()
        .find(|pack| pack.id == id)
        .map(|pack| pack.name.clone())
}

pub fn load_theme_packs(directory: impl AsRef<Path>) -> Vec<String> {
    let mut problems = Vec::new();
    let mut found = Vec::new();
    let mut folders: Vec<_> = std::fs::read_dir(directory.as_ref())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("theme.json").is_file())
        .collect();
    folders.sort_by_key(|entry| entry.file_name());
    for entry in folders.into_iter().take(PACK_LIMIT) {
        let id = entry.file_name().to_string_lossy().into_owned();
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            problems.push(format!(
                "{id}: folder names may use letters, digits, - and _"
            ));
            continue;
        }
        let source = match std::fs::read_to_string(entry.path().join("theme.json")) {
            Ok(source) => source,
            Err(error) => {
                problems.push(format!("{id}: {error}"));
                continue;
            }
        };
        let (theme, report) = parse_user_theme_in(&source, Some(&entry.path()));
        problems.extend(report.rejected.iter().map(|entry| format!("{id}: {entry}")));
        found.push(Pack {
            name: theme.name.clone().unwrap_or_else(|| id.clone()),
            base: theme
                .base
                .clone()
                .unwrap_or_else(|| crate::theme::DEFAULT_THEME.to_owned()),
            id,
            theme,
        });
    }
    if let Ok(mut packs) = PACKS.write() {
        *packs = found;
    }
    crate::theme::refresh_choices();
    activate(&crate::theme::selected_theme_key());
    crate::theme::invalidate_active();
    problems
}

pub fn set_user_theme_from_path(path: impl AsRef<Path>) -> Result<UserThemeReport, ThemeError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|error| ThemeError::InvalidSource(format!("{}: {error}", path.display())))?;
    let (theme, report) = parse_user_theme_in(&source, path.parent());
    if let Ok(mut file) = FILE_THEME.write() {
        *file = Some(theme);
    }
    activate(&crate::theme::selected_theme_key());
    crate::theme::invalidate_active();
    Ok(report)
}

pub fn active_fonts() -> Vec<(&'static str, Arc<Vec<u8>>)> {
    user_theme()
        .read()
        .map(|user| user.fonts.clone())
        .unwrap_or_default()
}

pub fn active_background() -> Option<Arc<Vec<u8>>> {
    user_theme()
        .read()
        .ok()
        .and_then(|user| user.background.clone())
}

pub fn assets_key() -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Ok(user) = user_theme().read() {
        for (weight, bytes) in &user.fonts {
            weight.hash(&mut hasher);
            bytes.hash(&mut hasher);
        }
        user.background.hash(&mut hasher);
    }
    hasher.finish()
}

fn load_asset(
    base: Option<&Path>,
    relative: &str,
    extensions: &[&str],
    limit: u64,
) -> Result<Arc<Vec<u8>>, String> {
    let base = base.ok_or("assets need a theme folder")?;
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("expected a relative path inside the theme folder".to_owned());
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !extensions.contains(&extension.as_str()) {
        return Err(format!("expected a {} file", extensions.join("/")));
    }
    let full = base.join(path);
    let size = std::fs::metadata(&full)
        .map_err(|error| error.to_string())?
        .len();
    if size > limit {
        return Err(format!("file is larger than {} MB", limit / 1024 / 1024));
    }
    std::fs::read(&full)
        .map(Arc::new)
        .map_err(|error| error.to_string())
}

pub fn clear_user_theme() {
    if let Ok(mut file) = FILE_THEME.write() {
        *file = None;
    }
    activate(&crate::theme::selected_theme_key());
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

pub fn active_vars() -> Vec<(String, String)> {
    let Ok(user) = user_theme().read() else {
        return Vec::new();
    };
    user.css_vars()
}

fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

impl UserTheme {
    fn css_vars(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();
        for (name, value) in &self.colors {
            vars.push((format!("color-{}", kebab(name)), value.clone()));
        }
        if let Some(enabled) = self.motion_enabled {
            vars.push(("motion-enabled".to_owned(), u8::from(enabled).to_string()));
        }
        for (name, value) in &self.motion_ms {
            vars.push((format!("motion-{}", kebab(name)), value.to_string()));
        }
        for (name, value) in &self.vars {
            vars.push((name.clone(), value.to_string()));
        }
        vars
    }
}

pub fn parse_user_theme(source: &str) -> (UserTheme, UserThemeReport) {
    parse_user_theme_in(source, None)
}

pub fn parse_user_theme_in(source: &str, base: Option<&Path>) -> (UserTheme, UserThemeReport) {
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
            "schemaVersion" => {}
            "name" => match body.as_str().map(str::trim).filter(|name| !name.is_empty()) {
                Some(name) => theme.name = Some(name.chars().take(40).collect()),
                None => report.rejected.push("name: expected text".to_owned()),
            },
            "base" => match body.as_str().filter(|id| crate::theme::is_builtin(id)) {
                Some(id) => theme.base = Some(id.to_owned()),
                None => report
                    .rejected
                    .push("base: unknown built-in theme".to_owned()),
            },
            "colors" => parse_colors(&mut theme, body, &mut report),
            "shape" => {
                theme.shape = numeric_section("shape", body, &SHAPE_KEYS, 0.0, 64.0, &mut report);
            }
            "spacing" => {
                theme.spacing =
                    numeric_section("spacing", body, &SPACING_KEYS, 0.0, 96.0, &mut report);
            }
            "motion" => parse_motion(&mut theme, body, &mut report),
            "fonts" => parse_fonts(&mut theme, body, base, &mut report),
            "images" => parse_images(&mut theme, body, base, &mut report),
            "vars" => parse_vars(&mut theme, body, &mut report),
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

fn parse_fonts(
    theme: &mut UserTheme,
    body: &Value,
    base: Option<&Path>,
    report: &mut UserThemeReport,
) {
    for (key, value) in entries("fonts", body, report) {
        let Some(weight) = FONT_WEIGHTS.iter().find(|known| **known == key) else {
            report.rejected.push(format!("fonts.{key}: unknown weight"));
            continue;
        };
        let loaded = value
            .as_str()
            .ok_or_else(|| "expected a file path".to_owned())
            .and_then(|path| load_asset(base, path, &["ttf", "otf"], FONT_LIMIT))
            .and_then(|bytes| {
                if skrifa::FontRef::new(&bytes).is_ok() {
                    Ok(bytes)
                } else {
                    Err("not a valid font".to_owned())
                }
            });
        match loaded {
            Ok(bytes) => {
                theme.fonts.retain(|(existing, _)| existing != weight);
                theme.fonts.push((weight, bytes));
                report.applied += 1;
            }
            Err(error) => report.rejected.push(format!("fonts.{key}: {error}")),
        }
    }
}

fn parse_images(
    theme: &mut UserTheme,
    body: &Value,
    base: Option<&Path>,
    report: &mut UserThemeReport,
) {
    for (key, value) in entries("images", body, report) {
        if key != "background" {
            report.rejected.push(format!("images.{key}: unknown image"));
            continue;
        }
        let loaded = value
            .as_str()
            .ok_or_else(|| "expected a file path".to_owned())
            .and_then(|path| load_asset(base, path, &["png", "jpg", "jpeg", "webp"], IMAGE_LIMIT));
        match loaded {
            Ok(bytes) => {
                theme.background = Some(bytes);
                report.applied += 1;
            }
            Err(error) => report.rejected.push(format!("images.{key}: {error}")),
        }
    }
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

fn parse_vars(theme: &mut UserTheme, body: &Value, report: &mut UserThemeReport) {
    for (key, value) in entries("vars", body, report) {
        let valid = key
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            && !key.is_empty();
        if !valid {
            report
                .rejected
                .push(format!("vars.{key}: expected a lowercase dashed name"));
            continue;
        }
        match number(value, -100000.0, 100000.0) {
            Some(number) => {
                theme.vars.retain(|(existing, _)| existing != key);
                theme.vars.push((key.clone(), number));
                report.applied += 1;
            }
            None => report
                .rejected
                .push(format!("vars.{key}: expected a number")),
        }
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
    fn overrides_become_css_variables() {
        let (user, report) = parse_user_theme(
            r##"{"colors": {"accent": "#112233", "buttonPrimary": "#E8A33D"},
                "motion": {"enabled": false, "fastMs": 50},
                "vars": {"focused-scale": 1.2, "Bad Name": 3}}"##,
        );
        assert_eq!(report.rejected.len(), 1, "{:?}", report.rejected);
        let vars = user.css_vars();
        let find = |name: &str| {
            vars.iter()
                .find(|(key, _)| key == name)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(find("color-accent"), Some("#112233FF"));
        assert_eq!(find("color-button-primary"), Some("#E8A33DFF"));
        assert_eq!(find("motion-enabled"), Some("0"));
        assert_eq!(find("motion-fast-ms"), Some("50"));
        assert_eq!(find("focused-scale"), Some("1.2"));
    }

    #[test]
    fn theme_packs_load_from_folders_and_can_be_selected() {
        let _guard = crate::GLOBAL_STATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/theme-pack-test");
        let _ = std::fs::remove_dir_all(&root);
        let write = |folder: &str, json: &str| {
            let dir = root.join(folder);
            std::fs::create_dir_all(&dir).expect("dir");
            std::fs::write(dir.join("theme.json"), json).expect("write");
        };
        write(
            "warm",
            r##"{"name": "Warm", "base": "amoled", "colors": {"accent": "#E8A33D"},
                "home": {"navItems": ["home"]}}"##,
        );
        write("not valid!", "{}");

        let problems = load_theme_packs(&root);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(crate::theme::choice_keys().contains(&"pack:warm"));
        assert!(crate::theme::choice_keys().contains(&"amoled"));

        crate::theme::set_selected_theme("pack:warm");
        assert_eq!(crate::theme::selected_theme_id(), "amoled");
        assert_eq!(pack_name("pack:warm").as_deref(), Some("Warm"));
        assert_eq!(nav_items(), Some(vec!["home".to_owned()]));

        crate::theme::set_selected_theme(crate::theme::DEFAULT_THEME);
        assert_eq!(nav_items(), None);

        let _ = std::fs::remove_dir_all(&root);
        load_theme_packs(&root);
        assert!(!crate::theme::choice_keys().contains(&"pack:warm"));
    }

    #[test]
    fn pack_fonts_and_images_are_loaded_and_validated() {
        let _guard = crate::GLOBAL_STATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/theme-asset-test");
        let _ = std::fs::remove_dir_all(&root);
        let pack = root.join("styled");
        std::fs::create_dir_all(pack.join("fonts")).expect("dir");
        std::fs::create_dir_all(pack.join("images")).expect("dir");
        let font = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../apps/android/app/src/main/res/font/archivo_regular.ttf"),
        )
        .expect("font");
        std::fs::write(pack.join("fonts/body.ttf"), &font).expect("write");
        std::fs::write(pack.join("fonts/broken.ttf"), b"not a font").expect("write");
        std::fs::write(pack.join("images/bg.jpg"), b"jpeg bytes").expect("write");
        std::fs::write(
            pack.join("theme.json"),
            r#"{"name": "Styled",
                "fonts": {"regular": "fonts/body.ttf", "bold": "fonts/broken.ttf",
                          "medium": "../escape.ttf", "semibold": "/etc/hosts.ttf"},
                "images": {"background": "images/bg.jpg", "logo": "images/x.png"}}"#,
        )
        .expect("write");

        let problems = load_theme_packs(&root);
        assert_eq!(problems.len(), 4, "{problems:?}");

        crate::theme::set_selected_theme("pack:styled");
        let fonts = active_fonts();
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].0, "regular");
        assert_eq!(
            active_background().as_deref().map(Vec::as_slice),
            Some(&b"jpeg bytes"[..])
        );
        let key = assets_key();

        crate::theme::set_selected_theme(crate::theme::DEFAULT_THEME);
        assert!(active_fonts().is_empty());
        assert_ne!(assets_key(), key);

        let _ = std::fs::remove_dir_all(&root);
        load_theme_packs(&root);
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
