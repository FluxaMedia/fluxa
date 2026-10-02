use regex::Regex;
use std::sync::LazyLock;

mod rules;
mod theme;
#[cfg(test)]
mod tests;

pub use theme::Theme;
pub use rules::{
    BadgeLook, Filter, MAX_PACKS, Pack, Rules, compile, parse_color, parse_pack, valid_pattern,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamBadge {
    pub kind: BadgeKind,
    pub label: String,
    pub look: BadgeLook,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub built_in: bool,
    pub file_size: bool,
    pub theme: Theme,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            built_in: true,
            file_size: true,
            theme: Theme::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Resolution,
    Source,
    Hdr,
    Codec,
    Audio,
    Size,
    Seeders,
    Custom,
}

struct Rule {
    kind: BadgeKind,
    label: &'static str,
    pattern: Regex,
}

fn rule(kind: BadgeKind, label: &'static str, pattern: &str) -> Rule {
    Rule {
        kind,
        label,
        pattern: Regex::new(&format!("(?i){pattern}")).expect("valid badge pattern"),
    }
}

static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    use BadgeKind::*;
    vec![
        rule(Resolution, "4K", r"\b(4k|2160p|uhd)\b"),
        rule(Resolution, "1440p", r"\b1440p\b"),
        rule(Resolution, "1080p", r"\b1080[pi]\b"),
        rule(Resolution, "720p", r"\b720p\b"),
        rule(Resolution, "480p", r"\b(480p|576p)\b"),
        rule(Source, "REMUX", r"\bremux\b"),
        rule(Source, "BluRay", r"\b(blu-?ray|bdrip|brrip)\b"),
        rule(Source, "WEB-DL", r"\bweb[ .-]?dl\b"),
        rule(Source, "WEBRip", r"\bweb-?rip\b"),
        rule(Source, "HDTV", r"\bhdtv\b"),
        rule(Source, "DVD", r"\bdvd(rip)?\b"),
        rule(Source, "CAM", r"\b(cam|camrip|hdcam|telesync|ts|tc)\b"),
        rule(Hdr, "DV", r"\b(dv|dovi|dolby[ .-]?vision)\b"),
        rule(Hdr, "HDR10+", r"\bhdr10\+|\bhdr10plus\b"),
        rule(Hdr, "HDR10", r"\bhdr10\b(?:[^+]|$)"),
        rule(Hdr, "HDR", r"\bhdr\b"),
        rule(Hdr, "10bit", r"\b10[ .-]?bit\b"),
        rule(Codec, "HEVC", r"\b(hevc|x265|h[ .]?265)\b"),
        rule(Codec, "AV1", r"\bav1\b"),
        rule(Codec, "AVC", r"\b(avc|x264|h[ .]?264)\b"),
        rule(Audio, "Atmos", r"\batmos\b"),
        rule(Audio, "TrueHD", r"\btrue-?hd\b"),
        rule(Audio, "DTS-X", r"\bdts[ .:-]?x\b"),
        rule(Audio, "DTS-HD", r"\bdts[ .-]?hd(?:[ .-]?ma)?\b"),
        rule(Audio, "DTS", r"\bdts\b"),
        rule(Audio, "DD+", r"\b(dd\+|ddp|e-?ac-?3|dolby digital plus)"),
        rule(Audio, "DD", r"\b(ac-?3|dd|dolby digital)\b"),
        rule(Audio, "AAC", r"\baac\b"),
        rule(Audio, "7.1", r"\b7[ .]1\b"),
        rule(Audio, "5.1", r"\b5[ .]1\b"),
    ]
});

static SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*(gb|gib|mb|mib)\b").unwrap());
static SEEDERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:👤|\bseeders?\b|\bseeds?\b)\s*:?\s*(\d+)").unwrap());

pub fn parse(
    text: &str,
    video_size: Option<i64>,
    options: Options,
    rules: &Rules,
) -> Vec<StreamBadge> {
    let mut badges: Vec<StreamBadge> = Vec::new();

    for rule in RULES.iter().filter(|_| options.built_in) {
        if rule.pattern.is_match(text) {
            push(&mut badges, rule.kind, rule.label, options.theme);
        }
    }
    if badges
        .iter()
        .any(|badge| matches!(badge.label.as_str(), "HDR10+" | "HDR10"))
    {
        badges.retain(|badge| badge.label != "HDR");
    }
    if badges.iter().any(|badge| badge.label == "DTS-X") {
        badges.retain(|badge| !matches!(badge.label.as_str(), "DTS" | "DTS-HD"));
    } else if badges.iter().any(|badge| badge.label == "DTS-HD") {
        badges.retain(|badge| badge.label != "DTS");
    }
    if badges.iter().any(|badge| badge.label == "DD+") {
        badges.retain(|badge| badge.label != "DD");
    }
    let size = video_size
        .filter(|bytes| *bytes > 0)
        .map(format_bytes)
        .or_else(|| text_size(text));
    if let Some(size) = size.filter(|_| options.file_size) {
        push(&mut badges, BadgeKind::Size, &size, options.theme);
    }
    if options.built_in
        && let Some(seeders) = SEEDERS.captures(text).and_then(|captures| captures.get(1))
    {
        push(&mut badges, BadgeKind::Seeders, seeders.as_str(), options.theme);
    }
    badges.sort_by_key(|badge| badge.kind as u8);
    for filter in rules.active_filters().filter(|filter| filter.matches(text)) {
        if badges.iter().any(|badge| badge.label == filter.name) {
            continue;
        }
        badges.push(StreamBadge {
            kind: BadgeKind::Custom,
            label: filter.name.clone(),
            look: filter.look(),
            image_url: filter
                .image_url
                .clone()
                .filter(|url| !url.trim().is_empty()),
        });
    }
    badges
}

fn push(badges: &mut Vec<StreamBadge>, kind: BadgeKind, label: &str, theme: Theme) {
    if !badges.iter().any(|badge| badge.label == label) {
        badges.push(StreamBadge {
            kind,
            label: label.to_owned(),
            look: theme.look(kind),
            image_url: None,
        });
    }
}

fn text_size(text: &str) -> Option<String> {
    let captures = SIZE.captures(text)?;
    let unit = captures[2].to_ascii_uppercase();
    let unit = if unit.starts_with('G') { "GB" } else { "MB" };
    Some(format!("{} {unit}", captures[1].replace(',', ".")))
}

fn format_bytes(bytes: i64) -> String {
    let gb = bytes as f64 / 1_073_741_824.0;
    if gb >= 1.0 {
        format!("{gb:.2} GB")
    } else {
        format!("{:.0} MB", bytes as f64 / 1_048_576.0)
    }
}
