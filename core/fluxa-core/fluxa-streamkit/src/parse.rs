#![allow(clippy::expect_used)]

use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::release;
use crate::types::{
    AddonRef, ParsedFile, ParsedStream, RawStream, ServiceRef, StreamType, TorrentInfo,
};
use crate::vocab::{
    self, AUDIO_TAGS, CHANNELS, DUAL_AUDIO, ENCODES, LANGUAGES, MULTI, QUALITIES, RESOLUTIONS,
    SERVICES, VISUAL_TAGS, rx,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    NotPlayable,
    Addon(String),
}

pub enum ParseOutcome {
    Stream(Box<ParsedStream>),
    Skip,
}

pub struct ParseEnv {
    pub now_ms: i64,
}

static KNOWN_RES: [u32; 9] = [2160, 1440, 1080, 720, 576, 480, 360, 240, 144];

fn snap_resolution(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    if lower == "4k" {
        return Some("2160p".into());
    }
    let digits: String = lower.chars().take_while(char::is_ascii_digit).collect();
    let n: u32 = digits.parse().ok()?;
    let nearest = KNOWN_RES.iter().min_by_key(|r| r.abs_diff(n))?;
    Some(format!("{nearest}p"))
}

fn remove_title(text: &str, title: &str) -> String {
    let pattern = regex::escape(title)
        .replace("\\ ", "[. _]+")
        .replace(' ', "[. _]+");
    match Regex::new(&format!("(?i){pattern}")) {
        Ok(re) => re.replace(text, "").into_owned(),
        Err(_) => text.to_string(),
    }
}

fn dotted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dot = false;
    for c in text.chars() {
        if c.is_whitespace() {
            if !last_dot {
                out.push('.');
            }
            last_dot = true;
        } else {
            out.push(c);
            last_dot = false;
        }
    }
    out.trim_matches('.').to_string()
}

fn lang_name_for_code(code: &str) -> Option<String> {
    match code {
        "multi audio" => Some("Multi".into()),
        "dual audio" => Some("Dual Audio".into()),
        "multi subs" => None,
        iso => vocab::language_by_iso(iso).map(|l| l.name.to_string()),
    }
}

fn push_unique(list: &mut Vec<String>, value: String) {
    if !list.contains(&value) {
        list.push(value);
    }
}

pub fn parse_file(name: &str) -> ParsedFile {
    let stripped = vocab::strip_repost(name);
    let rel = release::parse(&stripped);
    let mut text = stripped.clone();
    if let Some(title) = rel.title.as_deref().filter(|t| t.chars().count() > 4) {
        text = remove_title(&text, title);
    }
    let text = dotted(&text);

    let mut file = ParsedFile {
        title: rel.title.clone(),
        year: rel.year.clone(),
        country: rel.country.clone(),
        episode_title: rel.episode_title.clone(),
        seasons: rel.seasons.clone(),
        episodes: rel.episodes.clone(),
        volumes: rel.volumes.clone(),
        date: rel.date.clone(),
        release_group: rel.group.clone(),
        editions: rel.editions.clone(),
        network: rel.network.clone(),
        site: rel.site.clone(),
        container: rel.container.clone(),
        extension: rel.extension.clone(),
        regraded: rel.regraded,
        repack: rel.repack,
        proper: rel.proper,
        uncensored: rel.uncensored,
        unrated: rel.unrated,
        upscaled: rel.upscaled,
        subbed: rel.subbed,
        dubbed: rel.dubbed,
        ..Default::default()
    };

    file.resolution = rel
        .resolution
        .as_deref()
        .and_then(snap_resolution)
        .or_else(|| vocab::first_match(&RESOLUTIONS, &text).map(|(e, _)| e.value.to_string()));
    file.quality = vocab::first_match(&QUALITIES, &text).map(|(e, _)| e.value.to_string());
    file.encode = vocab::first_match(&ENCODES, &text).map(|(e, _)| e.value.to_string());
    file.visual_tags = vocab::all_matches(&VISUAL_TAGS, &text)
        .into_iter()
        .map(|(e, _)| e.value.to_string())
        .collect();
    file.audio_tags = vocab::all_matches(&AUDIO_TAGS, &text)
        .into_iter()
        .map(|(e, _)| e.value.to_string())
        .collect();
    file.audio_channels = vocab::all_matches(&CHANNELS, &text)
        .into_iter()
        .map(|(e, _)| e.value.to_string())
        .collect();

    let mut lang_text = text.clone();
    if rel
        .group
        .as_deref()
        .is_some_and(|g| g.eq_ignore_ascii_case("ind"))
    {
        lang_text = rx(r"(?i)\bind\b").replace(&lang_text, "").into_owned();
    }
    let mut languages: Vec<String> = Vec::new();
    for lang in LANGUAGES.iter() {
        if lang.find_audio(&lang_text).is_some() {
            push_unique(&mut languages, lang.name.to_string());
        }
    }
    if MULTI.matches(&lang_text) {
        push_unique(&mut languages, "Multi".into());
    }
    if DUAL_AUDIO.matches(&lang_text) {
        push_unique(&mut languages, "Dual Audio".into());
    }
    for code in &rel.languages {
        if let Some(name) = lang_name_for_code(code) {
            push_unique(&mut languages, name);
        }
    }
    if languages.iter().any(|l| l == "Latino") {
        languages.retain(|l| l != "Spanish");
    }
    file.languages = languages;
    file.season_pack = !file.seasons.is_empty() && file.episodes.is_empty();
    file
}

fn union_front(folder: &[String], file: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for v in folder.iter().chain(file) {
        if !out.contains(v) {
            out.push(v.clone());
        }
    }
    out
}

fn populated(f: &ParsedFile) -> usize {
    [
        f.title.is_some(),
        f.year.is_some(),
        f.resolution.is_some(),
        f.quality.is_some(),
        f.encode.is_some(),
        !f.seasons.is_empty(),
        !f.episodes.is_empty(),
        f.release_group.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count()
}

pub fn merge_files(file: ParsedFile, folder: Option<ParsedFile>) -> ParsedFile {
    let Some(folder) = folder else { return file };
    let mut out = ParsedFile {
        year: file.year.clone().or(folder.year.clone()),
        country: file.country.clone().or(folder.country.clone()),
        episode_title: file.episode_title.clone().or(folder.episode_title.clone()),
        date: file.date.clone().or(folder.date.clone()),
        resolution: file.resolution.clone().or(folder.resolution.clone()),
        quality: file.quality.clone().or(folder.quality.clone()),
        encode: file.encode.clone().or(folder.encode.clone()),
        release_group: file.release_group.clone().or(folder.release_group.clone()),
        network: file.network.clone().or(folder.network.clone()),
        site: file.site.clone().or(folder.site.clone()),
        container: file.container.clone().or(folder.container.clone()),
        extension: file.extension.clone().or(folder.extension.clone()),
        regraded: file.regraded || folder.regraded,
        repack: file.repack || folder.repack,
        proper: file.proper || folder.proper,
        uncensored: file.uncensored || folder.uncensored,
        unrated: file.unrated || folder.unrated,
        upscaled: file.upscaled || folder.upscaled,
        subbed: file.subbed || folder.subbed,
        dubbed: file.dubbed || folder.dubbed,
        visual_tags: union_front(&folder.visual_tags, &file.visual_tags),
        audio_tags: union_front(&folder.audio_tags, &file.audio_tags),
        audio_channels: union_front(&folder.audio_channels, &file.audio_channels),
        languages: union_front(&folder.languages, &file.languages),
        subtitles: union_front(&folder.subtitles, &file.subtitles),
        editions: union_front(&folder.editions, &file.editions),
        ..Default::default()
    };
    let file_wins_seasons = !file.seasons.is_empty();
    if file_wins_seasons {
        out.seasons = file.seasons.clone();
        out.folder_seasons = folder.seasons.clone();
    } else {
        out.seasons = folder.seasons.clone();
    }
    if !file.episodes.is_empty() {
        out.episodes = file.episodes.clone();
        out.folder_episodes = folder.episodes.clone();
    } else {
        out.episodes = folder.episodes.clone();
    }
    out.volumes = if file.volumes.is_empty() {
        folder.volumes.clone()
    } else {
        file.volumes.clone()
    };
    out.title = if populated(&folder) > populated(&file) {
        folder.title.clone()
    } else {
        file.title.clone()
    };
    out.season_pack = file.season_pack
        || folder.season_pack
        || (!out.seasons.is_empty() && out.episodes.is_empty());
    out
}

pub fn parse_size_text(text: &str, base: f64) -> Option<u64> {
    static SIZE: LazyLock<Regex> = LazyLock::new(|| rx(r"(?i)(\d+(?:[.,]\d+)?)\s?([KMGT])i?B\b"));
    let c = SIZE.captures(text)?;
    let number: f64 = c.get(1)?.as_str().replace(',', ".").parse().ok()?;
    let power = match c.get(2)?.as_str().to_ascii_uppercase().as_str() {
        "K" => 1,
        "M" => 2,
        "G" => 3,
        _ => 4,
    };
    let bytes = (number * base.powi(power)).round();
    (bytes.is_finite() && bytes > 0.0).then_some(bytes as u64)
}

pub fn parse_duration_ms(text: &str) -> Option<u64> {
    static DUR: LazyLock<Regex> =
        LazyLock::new(|| rx(r"(?i)(?:^|[\s\[(])(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?(?:$|[\s\])])"));
    for c in DUR.captures_iter(text) {
        let part = |i: usize| c.get(i).and_then(|m| m.as_str().parse::<u64>().ok());
        let (h, m, s) = (part(1), part(2), part(3));
        if h.is_none() && m.is_none() && s.is_none() {
            continue;
        }
        let total = h.unwrap_or(0) * 3600 + m.unwrap_or(0) * 60 + s.unwrap_or(0);
        if total > 0 {
            return Some(total * 1000);
        }
    }
    None
}

pub fn parse_age_hours(text: &str) -> Option<f64> {
    static AGE: LazyLock<Regex> = LazyLock::new(|| rx(r"(?i)(\d+)\s?([mhdy])\b"));
    let c = AGE.captures(text)?;
    let n: f64 = c.get(1)?.as_str().parse().ok()?;
    let hours = match c.get(2)?.as_str().to_ascii_lowercase().as_str() {
        "m" => n / 60.0,
        "h" => n,
        "d" => n * 24.0,
        _ => n * 24.0 * 365.0,
    };
    Some(hours)
}

fn base32_to_hex(input: &str) -> Option<String> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut bits = String::new();
    for ch in input.to_ascii_uppercase().bytes() {
        let idx = alphabet.iter().position(|b| *b == ch)?;
        bits.push_str(&format!("{idx:05b}"));
    }
    let mut out = String::new();
    for chunk in bits.as_bytes().chunks(4) {
        if chunk.len() < 4 {
            break;
        }
        let nibble = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 2).ok()?;
        out.push(char::from_digit(u32::from(nibble), 16)?);
    }
    Some(out)
}

pub fn magnet_info_hash(url: &str) -> Option<String> {
    static MAGNET: LazyLock<Regex> =
        LazyLock::new(|| rx(r"(?i)btih:([0-9a-f]{40}|[a-z2-7]{32})(?:$|[&#])"));
    let c = MAGNET.captures(url)?;
    let raw = c.get(1)?.as_str();
    if raw.len() == 40 {
        Some(raw.to_lowercase())
    } else {
        base32_to_hex(raw)
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    out.push(b);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hash_from_url(url: &str) -> Option<String> {
    static HASH: LazyLock<Regex> =
        LazyLock::new(|| rx(r"(?i)(?:btih:|[-/\[(;:&])([0-9a-f]{40})(?:$|[-\])/:;&?])"));
    let decoded = percent_decode(url);
    HASH.captures(&decoded)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_lowercase())
}

fn strip_flair(line: &str) -> String {
    static EMOJI: LazyLock<Regex> =
        LazyLock::new(|| rx(r"^(?:[\p{Extended_Pictographic}\x{FE0F}\x{200D}\s])+"));
    static LABEL: LazyLock<Regex> = LazyLock::new(|| rx(r"^[^\s:]+:\s*"));
    let trimmed = line.trim();
    let no_emoji = EMOJI.replace(trimmed, "");
    LABEL.replace(no_emoji.trim(), "").trim().to_string()
}

fn region_flags(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pending: Option<char> = None;
    for c in text.chars() {
        if ('\u{1F1E6}'..='\u{1F1FF}').contains(&c) {
            match pending.take() {
                Some(first) => {
                    let flag: String = [first, c].iter().collect();
                    if let Some(l) = vocab::language_for_flag(&flag) {
                        if vocab::is_specific_language(l.name) {
                            push_unique(&mut out, l.name.to_string());
                        }
                    }
                }
                None => pending = Some(c),
            }
        } else {
            pending = None;
        }
    }
    out
}

pub trait AddonParser: Send + Sync {
    fn should_skip(&self, _raw: &RawStream) -> bool {
        false
    }

    fn filename_regex(&self) -> Option<&Regex> {
        None
    }

    fn folder_name_regex(&self) -> Option<&Regex> {
        None
    }

    fn size_base(&self) -> f64 {
        1024.0
    }

    fn seeders_regex(&self) -> Option<&Regex> {
        None
    }

    fn age_regex(&self) -> Option<&Regex> {
        None
    }

    fn get_error(&self, _raw: &RawStream) -> Option<(String, String)> {
        None
    }

    fn is_info_stream(&self, _raw: &RawStream) -> bool {
        false
    }

    fn get_message(&self, _raw: &RawStream) -> Option<String> {
        None
    }

    fn get_languages(&self, _raw: &RawStream) -> Vec<String> {
        Vec::new()
    }

    fn get_file_idx(&self, _raw: &RawStream) -> Option<u32> {
        None
    }

    fn is_private(&self, _raw: &RawStream) -> bool {
        false
    }

    fn is_freeleech(&self, _raw: &RawStream) -> bool {
        false
    }

    fn parse(
        &self,
        mut raw: RawStream,
        addon: &AddonRef,
        seq: &mut u32,
        env: &ParseEnv,
    ) -> Result<ParseOutcome, ParseError> {
        if self.should_skip(&raw) {
            return Ok(ParseOutcome::Skip);
        }
        if raw
            .description
            .as_deref()
            .is_none_or(|d| d.trim().is_empty())
        {
            raw.description = raw.title.clone();
        }
        if let Some(url) = raw.url.clone() {
            if url.starts_with("magnet:") {
                if raw.info_hash.is_none() {
                    raw.info_hash = magnet_info_hash(&url);
                }
                raw.url = None;
            }
        }
        let description = raw.description.clone().unwrap_or_default();
        let name = raw.name.clone().unwrap_or_default();

        static ACCOUNT_ERR: LazyLock<Regex> = LazyLock::new(|| {
            rx(r"(?i)invalid\s+\w+\s+(?:account|apikey|api key|token)|public rate limit exceeded")
        });
        if ACCOUNT_ERR.is_match(&description) {
            return Err(ParseError::Addon(description.trim().to_string()));
        }
        *seq += 1;
        let id = format!("{}:{}", addon.instance_id, seq);
        if let Some((title, body)) = self.get_error(&raw) {
            let mut s = ParsedStream::blank(id, addon.clone(), StreamType::Error);
            s.message = Some(format!("{title}: {body}"));
            return Ok(ParseOutcome::Stream(Box::new(s)));
        }
        if self.is_info_stream(&raw) {
            let mut s = ParsedStream::blank(id, addon.clone(), StreamType::Info);
            s.message = self.get_message(&raw).or(Some(description));
            return Ok(ParseOutcome::Stream(Box::new(s)));
        }
        if !raw.is_playable() {
            return Err(ParseError::NotPlayable);
        }

        let (filename, folder) = self.names(&raw, &description);
        let size = self.size(
            &raw,
            &description,
            &name,
            filename.as_deref(),
            folder.as_deref(),
        );
        let folder_size = raw
            .behavior_hints
            .folder_size
            .as_ref()
            .and_then(|v| match v {
                serde_json::Value::Number(n) => n.as_f64().map(|f| f.round() as u64),
                serde_json::Value::String(s) => parse_size_text(s, self.size_base()),
                _ => None,
            })
            .filter(|fs| *fs > 0)
            .filter(|fs| size.is_none_or(|s| (*fs as f64 - s as f64).abs() > s as f64 * 0.05));

        let seeders = raw
            .behavior_hints
            .seeders
            .filter(|s| *s >= 0.0)
            .map(|s| s as u32)
            .or_else(|| {
                static PEERS: LazyLock<Regex> = LazyLock::new(|| rx(r"[👥👤]\s*(\d+)"));
                let re = self.seeders_regex().unwrap_or(&PEERS);
                re.captures(&description)
                    .and_then(|c| c.get(1))
                    .and_then(|m| m.as_str().parse().ok())
            });

        let age = raw
            .behavior_hints
            .publish_date
            .as_deref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|d| {
                let hours =
                    (env.now_ms - d.with_timezone(&Utc).timestamp_millis()) as f64 / 3_600_000.0;
                hours.max(0.0)
            })
            .or_else(|| {
                self.age_regex()
                    .and_then(|re| re.captures(&description))
                    .and_then(|c| c.get(1))
                    .and_then(|m| parse_age_hours(m.as_str()))
            });

        let duration = raw
            .behavior_hints
            .duration
            .filter(|d| *d > 0.0)
            .map(|d| (d * 1000.0) as u64)
            .or_else(|| parse_duration_ms(&description));
        let bitrate = match (size, duration) {
            (Some(s), Some(d)) if d > 0 => {
                Some((s as f64 * 8.0 / (d as f64 / 1000.0)).round() as u64)
            }
            _ => None,
        };

        let indexer = raw.behavior_hints.indexer.clone().or_else(|| {
            static INDEXER: LazyLock<Regex> =
                LazyLock::new(|| rx(r"(?:🌐|⚙️|🔗|🔎|🔍|☁️)\s*([^\n\p{Extended_Pictographic}]+)"));
            INDEXER
                .captures(&description)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty())
        });

        let service = self.service(&raw, &name);
        let url_path = raw
            .url
            .as_deref()
            .map(|u| u.split(['?', '#']).next().unwrap_or(u).to_lowercase());
        let stream_type = if url_path
            .as_deref()
            .is_some_and(|p| p.ends_with(".m3u") || p.ends_with(".m3u8"))
        {
            StreamType::Live
        } else if raw.external_url.is_some() && raw.url.is_none() {
            StreamType::External
        } else if service.as_ref().is_some_and(|s| s.id == "easynews") {
            StreamType::Usenet
        } else if service.is_some() {
            StreamType::Debrid
        } else if raw.url.is_some() {
            StreamType::Http
        } else if raw.info_hash.is_some() {
            StreamType::P2p
        } else if raw.yt_id.is_some() {
            StreamType::Youtube
        } else if raw.nzb_url.is_some() {
            StreamType::StremioUsenet
        } else if raw.has_archive() {
            StreamType::Archive
        } else {
            return Err(ParseError::NotPlayable);
        };

        let file_parsed = filename.as_deref().map(parse_file).unwrap_or_default();
        let folder_parsed = folder.as_deref().map(parse_file);
        let mut parsed = merge_files(file_parsed, folder_parsed);
        self.apply_overrides(&mut parsed, &raw, &description, &name);
        if !parsed.season_pack {
            let wide = folder_size.zip(size).is_some_and(|(f, s)| f > 2 * s);
            if !parsed.episodes.is_empty() && (wide || parsed.episodes.len() > 5) {
                parsed.season_pack = true;
            }
        }

        let info_hash = raw
            .info_hash
            .clone()
            .map(|h| h.to_lowercase())
            .or_else(|| raw.url.as_deref().and_then(hash_from_url));
        let torrent = info_hash.map(|hash| TorrentInfo {
            info_hash: hash,
            file_idx: raw.file_idx.or_else(|| self.get_file_idx(&raw)),
            seeders,
            sources: raw.sources.clone(),
            private: self.is_private(&raw),
            freeleech: self.is_freeleech(&raw),
            file: filename.clone(),
            title: folder.clone().or_else(|| filename.clone()),
        });

        let mut s = ParsedStream::blank(id, addon.clone(), stream_type);
        s.url = raw.url.clone();
        s.nzb_url = raw.nzb_url.clone();
        s.external_url = raw.external_url.clone();
        s.yt_id = raw.yt_id.clone();
        s.subtitles = raw.subtitles.clone();
        if let Some(h) = &raw.behavior_hints.proxy_headers {
            s.request_headers = h.request.clone();
            s.response_headers = h.response.clone();
        }
        s.not_web_ready = raw.behavior_hints.not_web_ready;
        s.video_hash = raw.behavior_hints.video_hash.clone();
        s.filename = filename;
        s.folder_name = folder;
        s.size = size;
        s.folder_size = folder_size;
        s.indexer = indexer;
        s.service = service;
        s.duration = duration;
        s.age = age;
        s.bitrate = bitrate;
        s.message = self.get_message(&raw);
        s.library = addon.library;
        s.torrent = torrent;
        s.parsed = parsed;
        Ok(ParseOutcome::Stream(Box::new(s)))
    }

    fn names(&self, raw: &RawStream, description: &str) -> (Option<String>, Option<String>) {
        let filename = raw
            .behavior_hints
            .filename
            .clone()
            .or_else(|| {
                self.filename_regex()
                    .and_then(|re| re.captures(description))
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
            })
            .or_else(|| {
                let lines: Vec<&str> = description
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .take(5)
                    .collect();
                lines
                    .iter()
                    .find(|l| {
                        let probe = release::parse(&strip_flair(l));
                        probe.year.is_some()
                            || !probe.seasons.is_empty()
                            || !probe.episodes.is_empty()
                    })
                    .or_else(|| lines.first())
                    .map(|l| strip_flair(l))
            })
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty());

        let folder = self
            .folder_name_regex()
            .and_then(|re| re.captures(description))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .or_else(|| raw.behavior_hints.folder_name.clone())
            .filter(|f| !f.trim().is_empty());

        let folder = match (&filename, folder) {
            (Some(f), Some(d)) if vocab::normalise_key(f) == vocab::normalise_key(&d) => None,
            (_, d) => d,
        };
        (filename, folder)
    }

    fn size(
        &self,
        raw: &RawStream,
        description: &str,
        name: &str,
        filename: Option<&str>,
        folder: Option<&str>,
    ) -> Option<u64> {
        let positive = |v: Option<f64>| {
            v.filter(|f| f.is_finite() && *f > 0.0)
                .map(|f| f.round() as u64)
        };
        positive(raw.behavior_hints.video_size)
            .or_else(|| positive(raw.size))
            .or_else(|| positive(raw.size_bytes))
            .or_else(|| {
                let mut text = description.to_string();
                for part in [filename, folder].into_iter().flatten() {
                    text = text.replace(part, " ");
                }
                parse_size_text(&text, self.size_base())
            })
            .or_else(|| parse_size_text(name, self.size_base()))
    }

    fn service(&self, raw: &RawStream, name: &str) -> Option<ServiceRef> {
        if let Some(id) = raw.behavior_hints.service.as_deref() {
            if let Some(def) = vocab::service_by_id(id) {
                return Some(ServiceRef {
                    id: def.id.clone(),
                    cached: raw.behavior_hints.cached == Some(true),
                });
            }
        }
        let cleaned = rx(r"(?i)web-?dl").replace_all(name, "").into_owned();
        let matcher = SERVICES.iter().rev().find(|s| s.matches(&cleaned))?;
        let lower = cleaned.to_lowercase();
        let uncached = cleaned.contains('⏳')
            || cleaned.contains('☁')
            || lower.contains("download")
            || lower.contains("uncached");
        let cached = !uncached
            && (["⚡", "🚀", "🌩", "📫"].iter().any(|m| cleaned.contains(m))
                || lower.contains("cached")
                || rx(&format!(
                    r"(?i){}\+",
                    regex::escape(&matcher.def.short_name)
                ))
                .is_match(&cleaned));
        Some(ServiceRef {
            id: matcher.def.id.clone(),
            cached,
        })
    }

    fn apply_overrides(
        &self,
        parsed: &mut ParsedFile,
        raw: &RawStream,
        description: &str,
        name: &str,
    ) {
        let mut flagged = region_flags(description);
        for l in region_flags(name) {
            push_unique(&mut flagged, l);
        }
        for l in self.get_languages(raw) {
            push_unique(&mut flagged, l);
        }
        for l in flagged {
            push_unique(&mut parsed.languages, l);
        }
    }
}

pub struct DefaultParser;

impl AddonParser for DefaultParser {}

pub fn parser_for(_preset_id: &str) -> Box<dyn AddonParser> {
    Box::new(DefaultParser)
}
