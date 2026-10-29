use serde_json::Value;

use crate::types::{MediaInfoQuality, ParsedStream, Track};
use crate::vocab::language_by_iso2;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Probed {
    pub audio: Vec<Track>,
    pub subtitles: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeError {
    Json(String),
    NoStreams,
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

fn channels(stream: &Value) -> Option<String> {
    let layout = match stream.get("channels")?.as_u64()? {
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
}
