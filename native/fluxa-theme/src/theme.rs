use std::{
    fmt,
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::Color;

pub const BUILTIN_THEMES: [&str; 3] = ["fluxa-dark", "amoled", "midnight-blue"];

pub const DEFAULT_THEME: &str = "fluxa-dark";

#[derive(Clone, Debug, PartialEq)]
pub enum ThemeError {
    InvalidSource(String),
    InvalidColor { token: String, value: String },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource(error) => write!(formatter, "invalid theme: {error}"),
            Self::InvalidColor { token, value } => {
                write!(formatter, "invalid color token {token}: {value}")
            }
        }
    }
}

impl std::error::Error for ThemeError {}

static EPOCH: AtomicU64 = AtomicU64::new(0);
static SELECTED: RwLock<Option<(String, String)>> = RwLock::new(None);
static CHOICES: RwLock<Option<&'static [&'static str]>> = RwLock::new(None);

pub fn is_builtin(id: &str) -> bool {
    BUILTIN_THEMES.contains(&id)
}

pub fn name_key(id: &str) -> String {
    format!("theme.{}", id.replace('-', "_"))
}

pub fn selected_theme_key() -> String {
    SELECTED
        .read()
        .ok()
        .and_then(|selected| selected.as_ref().map(|(key, _)| key.clone()))
        .unwrap_or_else(|| DEFAULT_THEME.to_owned())
}

pub fn selected_theme_id() -> String {
    SELECTED
        .read()
        .ok()
        .and_then(|selected| selected.as_ref().map(|(_, base)| base.clone()))
        .unwrap_or_else(|| DEFAULT_THEME.to_owned())
}

pub fn set_selected_theme(key: &str) {
    if selected_theme_key() == key {
        return;
    }
    let base = match crate::user::pack_base(key) {
        Some(base) => base,
        None if is_builtin(key) => key.to_owned(),
        None => return,
    };
    if let Ok(mut selected) = SELECTED.write() {
        *selected = Some((key.to_owned(), base));
    }
    crate::user::activate(key);
    invalidate_active();
}

pub fn choice_keys() -> &'static [&'static str] {
    if let Some(keys) = CHOICES.read().ok().and_then(|keys| *keys) {
        return keys;
    }
    refresh_choices()
}

pub(crate) fn refresh_choices() -> &'static [&'static str] {
    let mut keys: Vec<&'static str> = BUILTIN_THEMES.to_vec();
    keys.extend(
        crate::user::pack_keys()
            .into_iter()
            .map(|key| &*Box::leak(key.into_boxed_str())),
    );
    let keys: &'static [&'static str] = Box::leak(keys.into_boxed_slice());
    if let Ok(mut choices) = CHOICES.write() {
        *choices = Some(keys);
    }
    keys
}

pub fn active_epoch() -> u64 {
    EPOCH.load(Ordering::Acquire)
}

pub fn invalidate_active() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
}

pub fn parse_hex_color(token: &str, value: &str) -> Result<Color, ThemeError> {
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
    fn selecting_a_theme_ignores_unknown_ids() {
        let _guard = crate::GLOBAL_STATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        set_selected_theme("amoled");
        assert_eq!(selected_theme_id(), "amoled");
        set_selected_theme("no-such-theme");
        assert_eq!(selected_theme_id(), "amoled");
        set_selected_theme(DEFAULT_THEME);
        assert_eq!(selected_theme_id(), DEFAULT_THEME);
    }

    #[test]
    fn hex_tokens_convert_to_unit_rgb() {
        let color = parse_hex_color("accent", "#E85D3F").expect("color");
        assert_eq!(color, Color::rgb(232.0 / 255.0, 93.0 / 255.0, 63.0 / 255.0));
    }

    #[test]
    fn name_keys_follow_the_translation_scheme() {
        assert_eq!(name_key("midnight-blue"), "theme.midnight_blue");
    }
}
