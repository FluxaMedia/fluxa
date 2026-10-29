use std::collections::HashMap;

use serde_json::Value;

use crate::types::{MediaInfoQuality, ParsedStream, Track};
use crate::vocab::{language_by_iso, language_by_iso2};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Probed {
    pub audio: Vec<Track>,
    pub subtitles: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeError {
    Json(String),
    NoStreams,
    NoUrl,
    Tool(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeFormat {
    Ffprobe,
    MediaInfo,
}

pub trait Prober {
    fn format(&self) -> ProbeFormat;
    fn run(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> impl Future<Output = Result<String, String>>;
}

fn tag<'a>(stream: &'a Value, key: &str) -> Option<&'a str> {
    stream
        .get("tags")?
        .as_object()?
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .and_then(|(_, v)| v.as_str())
        .filter(|s| !s.trim().is_empty())
}

fn flag(stream: &Value, key: &str) -> bool {
    stream
        .get("disposition")
        .and_then(|d| d.get(key))
        .and_then(Value::as_i64)
        .is_some_and(|v| v != 0)
}

fn language(code: &str) -> Option<String> {
    if matches!(code, "und" | "mis" | "zxx" | "mul") {
        return None;
    }
    Some(language_by_iso2(code).map_or_else(|| code.to_string(), |l| l.name.to_string()))
}

fn channel_layout(count: u64) -> Option<String> {
    let layout = match count {
        1 => "1.0",
        2 => "2.0",
        3 => "2.1",
        4 => "4.0",
        5 => "5.0",
        6 => "5.1",
        7 => "6.1",
        8 => "7.1",
        _ => return None,
    };
    Some(layout.to_string())
}

fn channels(stream: &Value) -> Option<String> {
    channel_layout(stream.get("channels")?.as_u64()?)
}

fn audio_tags(stream: &Value) -> Vec<String> {
    let profile = stream
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();
    let mut tags = Vec::new();
    if profile.contains("atmos") {
        tags.push("Atmos".to_string());
    }
    if profile.contains("dts:x") || profile.contains("dts-x") {
        tags.push("DTS:X".to_string());
    }
    if profile.contains("dts-hd ma") {
        tags.push("DTS-HD MA".to_string());
    }
    tags
}

fn track(stream: &Value, audio: bool) -> Track {
    let title = tag(stream, "title").map(String::from);
    let commentary = flag(stream, "comment")
        || title
            .as_deref()
            .is_some_and(|t| t.to_lowercase().contains("commentary"));
    Track {
        lang: tag(stream, "language").and_then(language),
        title,
        codec: stream
            .get("codec_name")
            .and_then(Value::as_str)
            .map(String::from),
        channels: if audio { channels(stream) } else { None },
        tags: if audio {
            audio_tags(stream)
        } else {
            Vec::new()
        },
        default: flag(stream, "default"),
        forced: flag(stream, "forced"),
        hearing_impaired: flag(stream, "hearing_impaired"),
        commentary,
        dub: flag(stream, "dub"),
        original: flag(stream, "original"),
        visual_impaired: flag(stream, "visual_impaired"),
    }
}

pub fn parse_ffprobe(json: &str) -> Result<Probed, ProbeError> {
    let root: Value = serde_json::from_str(json).map_err(|e| ProbeError::Json(e.to_string()))?;
    let streams = root
        .get("streams")
        .and_then(Value::as_array)
        .filter(|s| !s.is_empty())
        .ok_or(ProbeError::NoStreams)?;
    let mut out = Probed::default();
    for s in streams {
        match s.get("codec_type").and_then(Value::as_str) {
            Some("audio") => out.audio.push(track(s, true)),
            Some("subtitle") => out.subtitles.push(track(s, false)),
            _ => {}
        }
    }
    Ok(out)
}

fn mi_text<'a>(track: &'a Value, key: &str) -> Option<&'a str> {
    track
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn mi_yes(track: &Value, key: &str) -> bool {
    mi_text(track, key).is_some_and(|v| v.eq_ignore_ascii_case("yes"))
}

fn mi_language(track: &Value) -> Option<String> {
    let raw = mi_text(track, "Language")?;
    let code = raw.split(['-', '_']).next().unwrap_or(raw).to_lowercase();
    if matches!(code.as_str(), "und" | "mis" | "zxx" | "mul") {
        return None;
    }
    let found = match code.len() {
        2 => language_by_iso(&code),
        3 => language_by_iso2(&code),
        _ => None,
    };
    Some(found.map_or_else(|| raw.to_string(), |l| l.name.to_string()))
}

fn mi_codec(format: &str) -> String {
    let lower = format.to_lowercase();
    match lower.as_str() {
        "mlp fba" | "mlp fba 16-ch" | "mlp fpe" => "truehd".to_string(),
        "pgs" => "hdmv_pgs_subtitle".to_string(),
        "utf-8" | "subrip" => "subrip".to_string(),
        "vobsub" => "dvd_subtitle".to_string(),
        other => other.replace(['-', ' '], ""),
    }
}

fn mi_track(track: &Value, audio: bool) -> Track {
    let title = mi_text(track, "Title").map(String::from);
    let lower_title = title.as_deref().unwrap_or_default().to_lowercase();
    let commercial = mi_text(track, "Format_Commercial_IfAny")
        .unwrap_or_default()
        .to_lowercase();
    let mut tags = Vec::new();
    if audio {
        if commercial.contains("atmos") {
            tags.push("Atmos".to_string());
        }
        if commercial.contains("dts:x") || commercial.contains("dts-x") {
            tags.push("DTS:X".to_string());
        }
        if commercial.contains("master audio") {
            tags.push("DTS-HD MA".to_string());
        }
    }
    Track {
        lang: mi_language(track),
        codec: mi_text(track, "Format").map(mi_codec),
        channels: if audio {
            mi_text(track, "Channels")
                .and_then(|c| c.parse::<u64>().ok())
                .and_then(channel_layout)
        } else {
            None
        },
        tags,
        default: mi_yes(track, "Default"),
        forced: mi_yes(track, "Forced"),
        hearing_impaired: lower_title.contains("sdh") || lower_title.contains("hearing impaired"),
        commentary: lower_title.contains("commentary"),
        dub: false,
        original: false,
        visual_impaired: lower_title.contains("audio description"),
        title,
    }
}

pub fn parse_mediainfo(json: &str) -> Result<Probed, ProbeError> {
    let root: Value = serde_json::from_str(json).map_err(|e| ProbeError::Json(e.to_string()))?;
    let tracks = root
        .get("media")
        .and_then(|m| m.get("track"))
        .and_then(Value::as_array)
        .filter(|t| !t.is_empty())
        .ok_or(ProbeError::NoStreams)?;
    let mut out = Probed::default();
    for t in tracks {
        match t.get("@type").and_then(Value::as_str) {
            Some("Audio") => out.audio.push(mi_track(t, true)),
            Some("Text") => out.subtitles.push(mi_track(t, false)),
            _ => {}
        }
    }
    Ok(out)
}

pub async fn probe_stream<P: Prober>(
    prober: &P,
    stream: &mut ParsedStream,
) -> Result<(), ProbeError> {
    let url = stream.url.clone().ok_or(ProbeError::NoUrl)?;
    let raw = prober
        .run(&url, &stream.request_headers)
        .await
        .map_err(ProbeError::Tool)?;
    let probed = match prober.format() {
        ProbeFormat::Ffprobe => parse_ffprobe(&raw)?,
        ProbeFormat::MediaInfo => parse_mediainfo(&raw)?,
    };
    apply(stream, probed);
    Ok(())
}

fn languages_of(tracks: &[Track]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for lang in tracks.iter().filter_map(|t| t.lang.as_ref()) {
        if !out.contains(lang) {
            out.push(lang.clone());
        }
    }
    out
}

pub fn apply(stream: &mut ParsedStream, probed: Probed) {
    let audio_langs = languages_of(&probed.audio);
    let sub_langs = languages_of(&probed.subtitles);
    if !audio_langs.is_empty() {
        stream.parsed.languages = audio_langs;
    }
    if !sub_langs.is_empty() {
        stream.parsed.subtitles = sub_langs;
    }
    stream.media_info_quality = Some(MediaInfoQuality::Probe);
    stream.audio_tracks = probed.audio;
    stream.subtitle_tracks = probed.subtitles;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AddonRef, StreamType};

    const SAMPLE: &str = r#"{"streams":[
        {"codec_type":"video","codec_name":"hevc"},
        {"codec_type":"audio","codec_name":"eac3","profile":"Dolby Digital Plus + Dolby Atmos","channels":6,
         "tags":{"LANGUAGE":"eng","title":"English 5.1"},"disposition":{"default":1,"forced":0}},
        {"codec_type":"audio","codec_name":"aac","channels":2,"tags":{"language":"fre"},"disposition":{"dub":1}},
        {"codec_type":"audio","codec_name":"aac","channels":2,"tags":{"language":"eng","title":"Director Commentary"}},
        {"codec_type":"subtitle","codec_name":"subrip","tags":{"language":"eng"},"disposition":{"forced":1}},
        {"codec_type":"subtitle","codec_name":"hdmv_pgs_subtitle","tags":{"language":"und"}}
    ]}"#;

    #[test]
    fn audio_tracks_carry_codec_channels_and_tags() {
        let p = parse_ffprobe(SAMPLE).unwrap();
        assert_eq!(p.audio.len(), 3);
        let first = &p.audio[0];
        assert_eq!(first.lang.as_deref(), Some("English"));
        assert_eq!(first.codec.as_deref(), Some("eac3"));
        assert_eq!(first.channels.as_deref(), Some("5.1"));
        assert_eq!(first.tags, vec!["Atmos".to_string()]);
        assert!(first.default);
    }

    #[test]
    fn bibliographic_codes_and_dispositions_map() {
        let p = parse_ffprobe(SAMPLE).unwrap();
        assert_eq!(p.audio[1].lang.as_deref(), Some("French"));
        assert!(p.audio[1].dub);
        assert!(p.audio[2].commentary);
        assert!(p.subtitles[0].forced);
        assert_eq!(p.subtitles[1].lang, None);
    }

    #[test]
    fn apply_marks_the_stream_as_probed_and_overrides_languages() {
        let mut s = ParsedStream::blank("x", AddonRef::default(), StreamType::P2p);
        s.parsed.languages = vec!["Multi".into()];
        apply(&mut s, parse_ffprobe(SAMPLE).unwrap());
        assert_eq!(s.media_info_quality, Some(MediaInfoQuality::Probe));
        assert_eq!(s.parsed.languages, vec!["English", "French"]);
        assert_eq!(s.parsed.subtitles, vec!["English"]);
    }

    #[test]
    fn empty_or_broken_output_is_an_error() {
        assert_eq!(
            parse_ffprobe(r#"{"streams":[]}"#),
            Err(ProbeError::NoStreams)
        );
        assert!(matches!(parse_ffprobe("nope"), Err(ProbeError::Json(_))));
    }

    const MEDIAINFO: &str = r#"{"media":{"track":[
        {"@type":"General"},
        {"@type":"Video","Format":"HEVC"},
        {"@type":"Audio","Format":"E-AC-3","Format_Commercial_IfAny":"Dolby Digital Plus with Dolby Atmos","Channels":"6","Language":"en","Default":"Yes","Forced":"No"},
        {"@type":"Audio","Format":"MLP FBA","Channels":"8","Language":"fra","Title":"Director Commentary"},
        {"@type":"Text","Format":"UTF-8","Language":"en-US","Forced":"Yes","Title":"English SDH"},
        {"@type":"Text","Format":"PGS","Language":"de"}
    ]}}"#;

    #[test]
    fn mediainfo_tracks_map_like_ffprobe_ones() {
        let p = parse_mediainfo(MEDIAINFO).unwrap();
        assert_eq!(p.audio.len(), 2);
        assert_eq!(p.audio[0].codec.as_deref(), Some("eac3"));
        assert_eq!(p.audio[0].channels.as_deref(), Some("5.1"));
        assert_eq!(p.audio[0].tags, vec!["Atmos".to_string()]);
        assert!(p.audio[0].default);
        assert_eq!(p.audio[1].lang.as_deref(), Some("French"));
        assert_eq!(p.audio[1].codec.as_deref(), Some("truehd"));
        assert!(p.audio[1].commentary);
        assert_eq!(p.subtitles[0].lang.as_deref(), Some("English"));
        assert!(p.subtitles[0].forced && p.subtitles[0].hearing_impaired);
        assert_eq!(p.subtitles[1].codec.as_deref(), Some("hdmv_pgs_subtitle"));
    }

    struct Canned(ProbeFormat, Result<String, String>);

    impl Prober for Canned {
        fn format(&self) -> ProbeFormat {
            self.0
        }

        async fn run(&self, _url: &str, _h: &HashMap<String, String>) -> Result<String, String> {
            self.1.clone()
        }
    }

    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = Box::pin(fut);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        loop {
            if let std::task::Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    fn with_url() -> ParsedStream {
        let mut s = ParsedStream::blank("x", AddonRef::default(), StreamType::Http);
        s.url = Some("https://x/a.mkv".into());
        s
    }

    #[test]
    fn probe_stream_runs_the_tool_and_applies_the_result() {
        let mut s = with_url();
        let prober = Canned(ProbeFormat::MediaInfo, Ok(MEDIAINFO.into()));
        block_on(probe_stream(&prober, &mut s)).unwrap();
        assert_eq!(s.media_info_quality, Some(MediaInfoQuality::Probe));
        assert_eq!(s.audio_tracks.len(), 2);
    }

    #[test]
    fn probe_stream_reports_missing_url_and_tool_failures() {
        let prober = Canned(ProbeFormat::Ffprobe, Err("not installed".into()));
        let mut no_url = ParsedStream::blank("x", AddonRef::default(), StreamType::P2p);
        assert_eq!(
            block_on(probe_stream(&prober, &mut no_url)),
            Err(ProbeError::NoUrl)
        );
        assert_eq!(
            block_on(probe_stream(&prober, &mut with_url())),
            Err(ProbeError::Tool("not installed".into()))
        );
    }
}
