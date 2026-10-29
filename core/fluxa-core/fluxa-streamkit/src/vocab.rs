#![allow(clippy::expect_used)]

use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

pub const UNKNOWN: &str = "Unknown";

pub fn rx(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static pattern")
}

pub struct Entry {
    pub value: &'static str,
    re: Regex,
    excl: Option<Regex>,
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Entry {
    fn new(value: &'static str, pat: &str, excl: Option<&str>) -> Self {
        let wrapped = format!(r"(?i)(?:^|[\s\[(_\-.,])(?P<t>{pat})(?:$|[\s\[()\]_\-.,])");
        Self {
            value,
            re: rx(&wrapped),
            excl: excl.map(|e| rx(&format!("(?i){e}"))),
        }
    }

    pub fn find(&self, text: &str) -> Option<Span> {
        self.find_where(text, |_| true)
    }

    pub fn find_where(&self, text: &str, accept: impl Fn(Span) -> bool) -> Option<Span> {
        if self.excl.as_ref().is_some_and(|e| e.is_match(text)) {
            return None;
        }
        self.re
            .captures_iter(text)
            .filter_map(|c| c.name("t"))
            .map(|m| Span {
                start: m.start(),
                end: m.end(),
            })
            .find(|s| accept(*s))
    }

    pub fn matches(&self, text: &str) -> bool {
        self.find(text).is_some()
    }
}

fn table(rows: &[(&'static str, &str, Option<&str>)]) -> Vec<Entry> {
    rows.iter()
        .map(|(value, pat, excl)| Entry::new(value, pat, *excl))
        .collect()
}

pub fn first_match<'a>(entries: &'a [Entry], text: &str) -> Option<(&'a Entry, Span)> {
    entries
        .iter()
        .find_map(|e| e.find(text).map(|span| (e, span)))
}

pub fn all_matches<'a>(entries: &'a [Entry], text: &str) -> Vec<(&'a Entry, Span)> {
    entries
        .iter()
        .filter_map(|e| e.find(text).map(|span| (e, span)))
        .collect()
}

pub static RESOLUTIONS: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        (
            "2160p",
            r"(?:bd|hd|m)?2160[pi]?|4k|uhd|ultra[ .-]?hd|3840x2160",
            None,
        ),
        ("1440p", r"(?:bd|hd|m)?1440[pi]?|2k|qhd|2560x1440", None),
        (
            "1080p",
            r"(?:bd|hd|m)?1080[pi]?|fhd|full[ .-]?hd|1920x1080",
            None,
        ),
        (
            "720p",
            r"(?:bd|hd|m)?720[pi]?|1280x720|hd",
            Some(r"hd[ .-]?(?:rip|tv|cam|ts|tc)"),
        ),
        ("576p", r"(?:bd|hd|m)?576[pi]?", None),
        ("480p", r"(?:bd|hd|m)?480[pi]?|sd|640x480|720x480", None),
        ("360p", r"(?:bd|hd|m)?360[pi]?", None),
        ("240p", r"(?:bd|hd|m)?240[pi]?", None),
        ("144p", r"(?:bd|hd|m)?144[pi]?", None),
    ])
});

pub static QUALITIES: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        (
            "BluRay REMUX",
            r"(?:(?:blu[ .-]?ray|bd|uhd)[ .-]?)?remux",
            Some(r"dvd[ .-]?remux"),
        ),
        (
            "BluRay",
            r"blu[ .-]?ray|br[ .-]?rip|bd[ .-]?rip|bdr|bd|uhd[ .-]?bd",
            Some(r"remux"),
        ),
        (
            "WEB-DL",
            r"web[ .-]?dl|web[ .-]?hd|web|amzn[ .-]?web",
            Some(r"web[ .-]?(?:dl[ .-]?)?(?:rip|cam)"),
        ),
        ("WEBRip", r"web[ .-]?(?:dl[ .-]?)?rip", None),
        ("HDRip", r"hd[ .-]?rip", Some(r"hc[ .-]?hd[ .-]?rip")),
        ("HC HD-Rip", r"hc[ .-]?hd[ .-]?rip", None),
        ("DVD REMUX", r"dvd[ .-]?remux", None),
        (
            "DVDRip",
            r"dvd[ .-]?rip|dvdr|dvd9|dvd5|dvd",
            Some(r"dvd[ .-]?(?:remux|scr)"),
        ),
        ("HDTV", r"hdtv|pdtv|tv[ .-]?rip|hdtv[ .-]?rip", None),
        ("CAM", r"cam(?:[ .-]?rip)?|hd[ .-]?cam", None),
        ("TS", r"tele[ .-]?sync|ts|hd[ .-]?ts|pdvd", None),
        ("TC", r"tele[ .-]?cine|tc|hd[ .-]?tc", None),
        ("SCR", r"scr(?:eener)?|dvd[ .-]?scr", None),
    ])
});

pub static ENCODES: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        ("AV1", r"av1", None),
        ("HEVC", r"hevc|[xh][ .-]?265", None),
        ("AVC", r"avc|[xh][ .-]?264", None),
        ("VC-1", r"vc-?1", None),
        ("XviD", r"xvid", None),
        ("DivX", r"divx|dvix", None),
        ("MPEG-4", r"mpeg-?4", None),
    ])
});

pub static VISUAL_TAGS: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        ("HDR10+", r"hdr10(?:\+|[ .-]?plus)", None),
        ("HDR10", r"hdr10", Some(r"hdr10(?:\+|[ .-]?plus)")),
        ("DV", r"dv|dovi|dolby[ .-]?vision", None),
        ("HDR", r"hdr", None),
        ("HLG", r"hlg", None),
        ("10bit", r"10[ .-]?bit|hi10p?", None),
        ("3D", r"3d", None),
        ("IMAX", r"imax", None),
        ("AI", r"ai[ .-]?(?:upscaled?|enhanced)", None),
        ("Upscaled", r"upscaled?", None),
        ("SDR", r"sdr", None),
        ("H-OU", r"h-?ou", None),
        ("H-SBS", r"h-?sbs", None),
    ])
});

pub static AUDIO_TAGS: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        ("Atmos", r"atmos", None),
        ("DD+", r"dd\+|ddp|e-?ac-?3|dd[ .-]?plus", None),
        (
            "DD",
            r"dd|ac-?3|dolby[ .-]?digital",
            Some(r"dd\+|ddp|e-?ac-?3|dd[ .-]?plus"),
        ),
        ("DTS:X", r"dts[ .:-]?x", None),
        ("DTS-HD MA", r"dts[ .-]?hd[ .-]?ma", None),
        ("DTS-HD", r"dts[ .-]?hd", Some(r"dts[ .-]?hd[ .-]?ma")),
        ("DTS-ES", r"dts[ .-]?es", None),
        ("DTS", r"dts", Some(r"dts[ .:-]?(?:x|hd|es)")),
        ("TrueHD", r"true[ .-]?hd", None),
        ("PCM", r"l?pcm", None),
        ("OPUS", r"opus", None),
        ("FLAC", r"flac", None),
        ("AAC", r"aac", None),
    ])
});

pub static CHANNELS: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    table(&[
        ("2.0", r"2[ .]0|stereo|2ch", None),
        ("5.1", r"5[ .]1(?:ch)?|6ch", None),
        ("6.1", r"6[ .]1", None),
        ("7.1", r"7[ .]1|8ch", None),
    ])
});

pub struct Language {
    pub name: &'static str,
    pub iso1: &'static str,
    pub iso2: &'static str,
    pub flag: &'static str,
    pub priority_flag: bool,
    entry: Entry,
}

impl Language {
    pub fn find_audio(&self, text: &str) -> Option<Span> {
        static SUB_AFTER: LazyLock<Regex> =
            LazyLock::new(|| rx(r"(?i)^[ ._-]?sub(?:s|titles?)?(?:$|[^a-z])"));
        self.entry.find_where(text, |s| {
            !SUB_AFTER.is_match(text.get(s.end..).unwrap_or(""))
        })
    }
}

fn lang(
    name: &'static str,
    iso1: &'static str,
    iso2: &'static str,
    flag: &'static str,
    priority_flag: bool,
    pat: &str,
) -> Language {
    Language {
        name,
        iso1,
        iso2,
        flag,
        priority_flag,
        entry: Entry::new(name, pat, None),
    }
}

pub static LANGUAGES: LazyLock<Vec<Language>> = LazyLock::new(|| {
    vec![
        lang("English", "en", "eng", "🇬🇧", true, r"eng(?:lish)?"),
        lang("Japanese", "ja", "jpn", "🇯🇵", true, r"jap(?:anese)?|jpn"),
        lang(
            "Chinese",
            "zh",
            "zho",
            "🇨🇳",
            true,
            r"chi(?:nese)?|chs|cht|mandarin|cantonese",
        ),
        lang("Russian", "ru", "rus", "🇷🇺", true, r"rus(?:sian)?"),
        lang("Arabic", "ar", "ara", "🇸🇦", true, r"ara(?:bic)?"),
        lang("Portuguese", "pt", "por", "🇵🇹", true, r"por(?:tuguese)?"),
        lang(
            "Portuguese (Brazil)",
            "pt-BR",
            "por",
            "🇧🇷",
            true,
            r"pt-?br|brazilian",
        ),
        lang(
            "Spanish",
            "es",
            "spa",
            "🇪🇸",
            true,
            r"spa(?:nish)?|esp|castellano",
        ),
        lang(
            "French",
            "fr",
            "fra",
            "🇫🇷",
            true,
            r"fre(?:nch)?|fra|vff?|truefrench",
        ),
        lang(
            "German",
            "de",
            "deu",
            "🇩🇪",
            true,
            r"ger(?:man)?|deu|deutsch",
        ),
        lang("Italian", "it", "ita", "🇮🇹", true, r"ita(?:lian)?"),
        lang("Korean", "ko", "kor", "🇰🇷", true, r"kor(?:ean)?"),
        lang("Hindi", "hi", "hin", "🇮🇳", true, r"hin(?:di)?"),
        lang("Bengali", "bn", "ben", "🇮🇳", false, r"bengali"),
        lang("Punjabi", "pa", "pan", "🇮🇳", false, r"punjabi"),
        lang("Marathi", "mr", "mar", "🇮🇳", false, r"marathi"),
        lang("Gujarati", "gu", "guj", "🇮🇳", false, r"gujarati"),
        lang("Tamil", "ta", "tam", "🇮🇳", false, r"tamil"),
        lang("Telugu", "te", "tel", "🇮🇳", false, r"telugu"),
        lang("Kannada", "kn", "kan", "🇮🇳", false, r"kannada"),
        lang("Malayalam", "ml", "mal", "🇮🇳", false, r"malayalam"),
        lang("Thai", "th", "tha", "🇹🇭", true, r"thai"),
        lang("Vietnamese", "vi", "vie", "🇻🇳", true, r"vie(?:tnamese)?"),
        lang("Indonesian", "id", "ind", "🇮🇩", true, r"ind(?:onesian)?"),
        lang("Turkish", "tr", "tur", "🇹🇷", true, r"tur(?:kish)?"),
        lang("Hebrew", "he", "heb", "🇮🇱", true, r"heb(?:rew)?"),
        lang("Persian", "fa", "fas", "🇮🇷", true, r"per(?:sian)?|farsi"),
        lang("Ukrainian", "uk", "ukr", "🇺🇦", true, r"ukr(?:ainian)?"),
        lang("Greek", "el", "ell", "🇬🇷", true, r"greek"),
        lang("Lithuanian", "lt", "lit", "🇱🇹", true, r"lithuanian"),
        lang("Latvian", "lv", "lav", "🇱🇻", true, r"latvian"),
        lang("Estonian", "et", "est", "🇪🇪", true, r"estonian"),
        lang("Polish", "pl", "pol", "🇵🇱", true, r"pol(?:ish)?"),
        lang("Czech", "cs", "ces", "🇨🇿", true, r"cze(?:ch)?|ces"),
        lang("Slovak", "sk", "slk", "🇸🇰", true, r"slovak|slk"),
        lang("Hungarian", "hu", "hun", "🇭🇺", true, r"hun(?:garian)?"),
        lang(
            "Romanian",
            "ro",
            "ron",
            "🇷🇴",
            true,
            r"rum(?:anian)?|ron|romanian",
        ),
        lang("Bulgarian", "bg", "bul", "🇧🇬", true, r"bulgarian"),
        lang("Serbian", "sr", "srp", "🇷🇸", true, r"serbian"),
        lang("Croatian", "hr", "hrv", "🇭🇷", true, r"croatian|hrv"),
        lang("Slovenian", "sl", "slv", "🇸🇮", true, r"slovenian|slv"),
        lang("Dutch", "nl", "nld", "🇳🇱", true, r"dut(?:ch)?|nld"),
        lang("Danish", "da", "dan", "🇩🇰", true, r"dan(?:ish)?"),
        lang("Finnish", "fi", "fin", "🇫🇮", true, r"fin(?:nish)?"),
        lang("Swedish", "sv", "swe", "🇸🇪", true, r"swe(?:dish)?"),
        lang("Norwegian", "no", "nor", "🇳🇴", true, r"nor(?:wegian)?"),
        lang("Malay", "ms", "msa", "🇲🇾", true, r"malay|msa"),
        lang("Latino", "es-419", "spa", "🇲🇽", false, r"latino|lat(?:am)?"),
    ]
});

pub static MULTI: LazyLock<Entry> = LazyLock::new(|| {
    Entry::new(
        "Multi",
        r"multi(?:[ .-]?(?:audio|lang(?:uages?)?))?",
        Some(r"multi[ .-]?sub"),
    )
});
pub static DUAL_AUDIO: LazyLock<Entry> =
    LazyLock::new(|| Entry::new("Dual Audio", r"dual[ .-]?audio|dual", None));
pub static DUBBED: LazyLock<Entry> =
    LazyLock::new(|| Entry::new("Dubbed", r"dubbed|dublado|dub", None));

pub fn language_by_name(name: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|l| l.name.eq_ignore_ascii_case(name))
}

pub fn language_by_iso(code: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|l| l.iso1.eq_ignore_ascii_case(code))
}

pub fn language_by_iso2(code: &str) -> Option<&'static Language> {
    let canonical = match code.to_ascii_lowercase().as_str() {
        "fre" => "fra".to_string(),
        "ger" => "deu".to_string(),
        "chi" => "zho".to_string(),
        "dut" => "nld".to_string(),
        "gre" => "ell".to_string(),
        "cze" => "ces".to_string(),
        "rum" => "ron".to_string(),
        "slo" => "slk".to_string(),
        "per" => "fas".to_string(),
        "may" => "msa".to_string(),
        "ice" => "isl".to_string(),
        "wel" => "cym".to_string(),
        "baq" => "eus".to_string(),
        "mac" => "mkd".to_string(),
        "alb" => "sqi".to_string(),
        "arm" => "hye".to_string(),
        "geo" => "kat".to_string(),
        "bur" => "mya".to_string(),
        other => other.to_string(),
    };
    LANGUAGES.iter().find(|l| l.iso2 == canonical)
}

pub fn language_for_flag(flag: &str) -> Option<&'static Language> {
    let mut hits = LANGUAGES.iter().filter(|l| l.flag == flag);
    let first = hits.next()?;
    if first.priority_flag {
        return Some(first);
    }
    std::iter::once(first)
        .chain(hits)
        .find(|l| l.priority_flag)
        .or(Some(first))
}

pub fn is_specific_language(name: &str) -> bool {
    !matches!(
        name,
        "Unknown" | "Dual Audio" | "Multi" | "Dubbed" | "Original"
    )
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDef {
    pub id: String,
    pub short_name: String,
    pub name: String,
    pub names: Vec<String>,
}

pub struct ServiceMatcher {
    pub def: ServiceDef,
    names: Vec<Regex>,
}

pub const SERVICES_JSON: &str = include_str!("../data/services.json");

pub static SERVICES: LazyLock<Vec<ServiceMatcher>> = LazyLock::new(|| {
    let defs: Vec<ServiceDef> = serde_json::from_str(SERVICES_JSON).unwrap_or_default();
    defs.into_iter()
        .map(|def| {
            let names = def
                .names
                .iter()
                .map(|n| {
                    rx(&format!(
                        r"(?i)(?:^|[\s|\[(_/\-.]){}(?:$|[\s⚡🚀🌩️📫⏳☁️+/|)\]_.\-])",
                        regex::escape(n)
                    ))
                })
                .collect();
            ServiceMatcher { def, names }
        })
        .collect()
});

impl ServiceMatcher {
    pub fn matches(&self, text: &str) -> bool {
        self.names.iter().any(|r| r.is_match(text))
    }
}

pub fn service_by_id(id: &str) -> Option<&'static ServiceDef> {
    SERVICES
        .iter()
        .map(|s| &s.def)
        .find(|d| d.id.eq_ignore_ascii_case(id))
}

pub const DEFAULT_REPOST_SUFFIXES: [&str; 9] = [
    "RP",
    "1",
    "NZBGeek",
    "Obfuscated",
    "Scrambled",
    "sample",
    "xpost",
    "postbot",
    "Rakuv*",
];

pub fn repost_regex(suffixes: &[&str]) -> Regex {
    let alt = suffixes
        .iter()
        .map(|s| match s.strip_suffix('*') {
            Some(stem) => format!("{}[A-Za-z0-9]*", regex::escape(stem)),
            None => regex::escape(s),
        })
        .collect::<Vec<_>>()
        .join("|");
    rx(&format!(r"(?i)(?:-(?:{alt}))+(\.[A-Za-z0-9]{{2,4}})?$"))
}

pub static REPOST: LazyLock<Regex> = LazyLock::new(|| repost_regex(&DEFAULT_REPOST_SUFFIXES));

pub fn strip_repost(name: &str) -> String {
    REPOST.replace(name, "$1").into_owned()
}

pub const VIDEO_EXTENSIONS: [&str; 14] = [
    "mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "m2ts", "mpg", "mpeg", "iso",
    "ogm",
];

pub fn split_video_extension(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once('.') {
        Some((stem, ext)) if VIDEO_EXTENSIONS.iter().any(|v| v.eq_ignore_ascii_case(ext)) => {
            (stem, Some(ext))
        }
        _ => (name, None),
    }
}

pub fn normalise_key(text: &str) -> String {
    let (stem, _) = split_video_extension(text.trim());
    stem.chars()
        .filter(|c| c.is_alphanumeric() || *c == '+')
        .flat_map(char::to_lowercase)
        .collect()
}
