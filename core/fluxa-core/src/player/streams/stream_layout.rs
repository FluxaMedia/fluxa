use regex::Regex;
use std::sync::LazyLock;

use super::stream_badges::{BadgeKind, Options, Rules, parse};

pub const PRESET_IDS: [&str; 3] = ["detailed", "compact", "minimal"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    pub name: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fields {
    pub resolution: String,
    pub source: String,
    pub hdr: String,
    pub codec: String,
    pub audio: String,
    pub size: String,
    pub seeders: String,
    pub languages: String,
}

pub fn preset(id: &str) -> Option<Layout> {
    let (name, detail) = match id {
        "detailed" => (
            "{resolution}",
            "🎬 {source} ➤ {codec} ➤ {hdr}\n🎧 {audio}\n📦 {size}\n👤 {seeders}\n🌐 {languages}",
        ),
        "compact" => (
            "{resolution} {hdr}",
            "{source} · {codec} · {audio}\n{size} · {languages}",
        ),
        "minimal" => ("{resolution} · {source}", "{size} · {languages}"),
        _ => return None,
    };
    Some(Layout {
        name: name.to_owned(),
        detail: detail.to_owned(),
    })
}

const LANGUAGES: &[(&str, &str)] = &[
    ("🌐", r"\bmulti(?:[ .-]?(?:audio|lang\w*))?\b"),
    ("🇬🇧", r"\b(?:english|eng)\b"),
    ("🇹🇷", r"\b(?:turkish|tur)\b"),
    ("🇩🇪", r"\b(?:german|ger|deutsch)\b"),
    ("🇫🇷", r"\b(?:french|fre|fra|vf|vostfr)\b"),
    ("🇪🇸", r"\b(?:spanish|spa|latino)\b"),
    ("🇮🇹", r"\b(?:italian|ita)\b"),
    ("🇵🇹", r"\b(?:portuguese|por)\b"),
    ("🇷🇺", r"\b(?:russian|rus)\b"),
    ("🇯🇵", r"\b(?:japanese|jpn|jap)\b"),
    ("🇰🇷", r"\b(?:korean|kor)\b"),
    ("🇨🇳", r"\b(?:chinese|chi|mandarin)\b"),
    ("🇮🇳", r"\b(?:hindi|hin)\b"),
    ("🇸🇦", r"\b(?:arabic|ara)\b"),
    ("🇵🇱", r"\b(?:polish|pol)\b"),
    ("🇳🇱", r"\b(?:dutch|nld)\b"),
];

static LANGUAGE_RULES: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    LANGUAGES
        .iter()
        .map(|(flag, pattern)| (*flag, Regex::new(&format!("(?i){pattern}")).unwrap()))
        .collect()
});

fn is_regional(c: char) -> bool {
    ('\u{1F1E6}'..='\u{1F1FF}').contains(&c)
}

fn languages(text: &str) -> String {
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((at, first)) = chars.next() {
        if is_regional(first)
            && let Some(&(_, second)) = chars.peek()
            && is_regional(second)
        {
            chars.next();
            found.push((at, format!("{first}{second}")));
        }
    }
    for (flag, regex) in LANGUAGE_RULES.iter() {
        if let Some(found_at) = regex.find(text) {
            found.push((found_at.start(), (*flag).to_owned()));
        }
    }
    found.sort_by_key(|(at, _)| *at);
    let mut flags: Vec<String> = Vec::new();
    for (_, flag) in found {
        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }
    flags.join(" ")
}

pub fn extract(text: &str, video_size: Option<i64>) -> Fields {
    let badges = parse(text, video_size, Options::default(), &Rules::default());
    let pick = |kind: BadgeKind, joiner: &str, only_first: bool| {
        let mut labels = badges
            .iter()
            .filter(|badge| badge.kind == kind)
            .map(|badge| badge.label.as_str());
        if only_first {
            labels.next().unwrap_or_default().to_owned()
        } else {
            labels.collect::<Vec<_>>().join(joiner)
        }
    };
    Fields {
        resolution: pick(BadgeKind::Resolution, "", true),
        source: pick(BadgeKind::Source, "", true),
        hdr: pick(BadgeKind::Hdr, " ", false),
        codec: pick(BadgeKind::Codec, "", true),
        audio: pick(BadgeKind::Audio, " | ", false),
        size: pick(BadgeKind::Size, "", true),
        seeders: pick(BadgeKind::Seeders, "", true),
        languages: languages(text),
    }
}

impl Fields {
    fn get(&self, name: &str) -> Option<&str> {
        Some(match name {
            "resolution" => &self.resolution,
            "source" => &self.source,
            "hdr" => &self.hdr,
            "codec" => &self.codec,
            "audio" => &self.audio,
            "size" => &self.size,
            "seeders" => &self.seeders,
            "languages" => &self.languages,
            _ => return None,
        })
    }
}

fn render_line(line: &str, fields: &Fields) -> String {
    let mut literals: Vec<String> = vec![String::new()];
    let mut values: Vec<&str> = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let value = fields.get(&rest[open + 1..open + close]);
        match value {
            Some(value) => {
                literals.last_mut().unwrap().push_str(&rest[..open]);
                values.push(value);
                literals.push(String::new());
            }
            None => literals
                .last_mut()
                .unwrap()
                .push_str(&rest[..open + close + 1]),
        }
        rest = &rest[open + close + 1..];
    }
    literals.last_mut().unwrap().push_str(rest);

    let mut out = String::new();
    let mut emitted = false;
    for (index, value) in values.iter().enumerate() {
        if value.is_empty() {
            continue;
        }
        if emitted {
            out.push_str(&literals[index]);
        } else if index == 0 {
            out.push_str(&literals[0]);
        }
        out.push_str(value);
        emitted = true;
    }
    if emitted {
        out.push_str(literals.last().map_or("", String::as_str));
        out.trim().to_owned()
    } else {
        String::new()
    }
}

pub fn render(template: &str, fields: &Fields) -> String {
    template
        .lines()
        .map(|line| render_line(line, fields))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const REMUX: &str = "Movie.2160p.UHD.BluRay.REMUX.HEVC.DV.HDR10.Atmos.TrueHD.7.1-GRP 🇬🇧 Italian";

    #[test]
    fn detailed_layout_matches_the_reference_card() {
        let layout = preset("detailed").unwrap();
        let fields = extract(REMUX, Some(67_645_734_912));
        assert_eq!(render(&layout.name, &fields), "4K");
        assert_eq!(
            render(&layout.detail, &fields),
            "🎬 REMUX ➤ HEVC ➤ DV HDR10\n🎧 Atmos | TrueHD | 7.1\n📦 63.00 GB\n🌐 🇬🇧 🇮🇹"
        );
    }

    #[test]
    fn missing_fields_drop_their_separators_and_empty_lines() {
        let fields = extract("Show S01E01 1080p WEB-DL x264", None);
        assert_eq!(
            render("🎬 {source} ➤ {codec} ➤ {hdr}\n🎧 {audio}", &fields),
            "🎬 WEB-DL ➤ AVC"
        );
        assert_eq!(render("{hdr} · {audio}", &fields), "");
    }

    #[test]
    fn unknown_placeholders_stay_literal() {
        let fields = extract("1080p", None);
        assert_eq!(render("{resolution} {nope}", &fields), "1080p {nope}");
    }
}
