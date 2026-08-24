use crate::types::resource::Stream;
use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalAudioRequest {
    #[serde(default)]
    streams: Vec<Stream>,
    selected_stream_url: Option<String>,
    preferred_audio_language: Option<String>,
}

struct Candidate {
    id: String,
    url: String,
    label: String,
    language: String,
    headers: Option<Value>,
    source_name: String,
    audio_only: bool,
}

fn source_name(stream: &Stream, index: usize) -> String {
    stream
        .name
        .as_deref()
        .or(stream.title.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Source {}", index + 1))
}

fn stream_language(stream: &Stream) -> String {
    stream
        .extra
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn headers_value(stream: &Stream) -> Option<Value> {
    stream
        .headers
        .as_ref()
        .filter(|headers| !headers.is_empty())
        .and_then(|headers| serde_json::to_value(headers).ok())
}

pub(crate) fn external_audio_options_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ExternalAudioRequest>(request_json).ok()?;
    let selected = request
        .selected_stream_url
        .as_deref()
        .filter(|url| !url.trim().is_empty());

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    if let Some(url) = selected {
        seen.insert(url.to_string());
    }

    for (index, stream) in request.streams.iter().enumerate() {
        let stream_url = stream.url.as_deref().filter(|url| !url.trim().is_empty());
        let is_selected = matches!((stream_url, selected), (Some(a), Some(b)) if a == b);
        let name = source_name(stream, index);
        let stream_headers = headers_value(stream);

        for track in stream.audio_tracks.iter().flatten() {
            if is_selected || track.url.trim().is_empty() || !seen.insert(track.url.clone()) {
                continue;
            }
            let headers = track
                .headers
                .as_ref()
                .filter(|headers| !headers.is_empty())
                .and_then(|headers| serde_json::to_value(headers).ok())
                .or_else(|| stream_headers.clone());
            candidates.push(Candidate {
                id: format!("ext-audio-{index}-{}", track.id),
                url: track.url.clone(),
                label: track.label.clone().unwrap_or_else(|| track.lang.clone()),
                language: track.lang.clone(),
                headers,
                source_name: name.clone(),
                audio_only: true,
            });
        }

        if is_selected {
            continue;
        }
        let Some(url) = stream_url else { continue };
        if stream.audio_tracks.iter().flatten().next().is_some() {
            continue;
        }
        if !seen.insert(url.to_string()) {
            continue;
        }
        let language = stream_language(stream);
        candidates.push(Candidate {
            id: format!("ext-audio-{index}-full"),
            url: url.to_string(),
            label: stream.title.clone().unwrap_or_else(|| name.clone()),
            language,
            headers: stream_headers,
            source_name: name,
            audio_only: false,
        });
    }

    let preferred = request
        .preferred_audio_language
        .as_deref()
        .map(str::to_lowercase)
        .filter(|value| !value.is_empty() && value != "none");

    let matches_preferred = |candidate: &Candidate| match preferred.as_deref() {
        Some(preferred) => super::language::subtitle_language_matches(
            &candidate.label,
            Some(&candidate.language),
            preferred,
        ),
        None => false,
    };

    let mut ordered: Vec<(usize, &Candidate)> = candidates.iter().enumerate().collect();
    ordered.sort_by_key(|(position, candidate)| {
        (
            !candidate.audio_only,
            !matches_preferred(candidate),
            *position,
        )
    });

    let options: Vec<Value> = ordered
        .into_iter()
        .map(|(_, candidate)| {
            json!({
                "id": candidate.id,
                "url": candidate.url,
                "label": candidate.label,
                "language": candidate.language,
                "headers": candidate.headers,
                "sourceName": candidate.source_name,
                "audioOnly": candidate.audio_only,
                "recommended": matches_preferred(candidate),
            })
        })
        .collect();

    serde_json::to_string(&json!({ "options": options })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn options(request: Value) -> Vec<Value> {
        let raw = external_audio_options_json(&request.to_string()).unwrap();
        serde_json::from_str::<Value>(&raw).unwrap()["options"]
            .as_array()
            .unwrap()
            .clone()
    }

    fn hdfilmizle_dual() -> Value {
        json!({
            "name": "HDFilmizle · Çift Dil",
            "title": "1080p · 1920x1080 · Çift Dil",
            "url": "https://cdn.example/master.m3u8",
            "language": "dual",
            "headers": { "Referer": "https://vidrame.pro/vr/397ac592" },
            "audioTracks": [
                { "id": "plugin-audio-0", "url": "https://cdn.example/audio-tur-1.m3u8", "lang": "tur", "label": "Türkçe" },
                { "id": "plugin-audio-1", "url": "https://cdn.example/audio-eng-2.m3u8", "lang": "eng", "label": "İngilizce" }
            ]
        })
    }

    #[test]
    fn a_stream_with_renditions_is_offered_only_through_them() {
        let result = options(json!({
            "streams": [
                { "name": "Torrent 2160p", "url": "http://127.0.0.1:8080/stream/0", "language": "en" },
                hdfilmizle_dual()
            ],
            "selectedStreamUrl": "http://127.0.0.1:8080/stream/0",
            "preferredAudioLanguage": "tr"
        }));

        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["url"], "https://cdn.example/audio-tur-1.m3u8");
        assert_eq!(result[0]["audioOnly"], true);
        assert_eq!(result[0]["recommended"], true);
        assert_eq!(result[1]["url"], "https://cdn.example/audio-eng-2.m3u8");
        assert_eq!(result[1]["recommended"], false);
    }

    #[test]
    fn rendition_inherits_stream_headers() {
        let result = options(json!({
            "streams": [hdfilmizle_dual()],
            "selectedStreamUrl": "http://127.0.0.1:8080/stream/0"
        }));

        assert_eq!(result[0]["headers"]["Referer"], "https://vidrame.pro/vr/397ac592");
        assert_eq!(result[0]["sourceName"], "HDFilmizle · Çift Dil");
    }

    #[test]
    fn renditions_of_the_selected_stream_stay_with_its_own_player() {
        let result = options(json!({
            "streams": [hdfilmizle_dual()],
            "selectedStreamUrl": "https://cdn.example/master.m3u8",
            "preferredAudioLanguage": "tr"
        }));

        assert!(result.is_empty());
    }

    #[test]
    fn selected_stream_is_never_offered_as_its_own_audio_source() {
        let result = options(json!({
            "streams": [
                { "name": "Torrent", "url": "http://127.0.0.1:8080/stream/0" },
                { "name": "HDFilmizle · Türkçe", "url": "https://cdn.example/tr.m3u8", "language": "tr" }
            ],
            "selectedStreamUrl": "http://127.0.0.1:8080/stream/0"
        }));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["url"], "https://cdn.example/tr.m3u8");
    }

    #[test]
    fn torrent_only_streams_are_skipped() {
        let result = options(json!({
            "streams": [
                { "name": "Selected", "url": "http://127.0.0.1:8080/stream/0" },
                { "name": "Magnet only", "infoHash": "abcdef" }
            ],
            "selectedStreamUrl": "http://127.0.0.1:8080/stream/0"
        }));

        assert!(result.is_empty());
    }
}
