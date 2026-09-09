use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioTrackCandidate {
    id: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    is_selected: bool,
    #[serde(default = "default_true")]
    is_supported: bool,
    #[serde(default)]
    channel_count: Option<i32>,
    #[serde(default)]
    sample_mime_type: Option<String>,
    #[serde(default)]
    bitrate: Option<i64>,
    #[serde(default)]
    sample_rate: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioTrackRequest {
    tracks_json: String,
    preferred_language: String,
    #[serde(default)]
    passthrough_track_ids_json: String,
    #[serde(default)]
    supported_sample_rates_json: String,
    #[serde(default)]
    max_pcm_channels: Option<i32>,
}

fn default_true() -> bool {
    true
}

pub(crate) fn select_audio_track_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<AudioTrackRequest>(request_json).ok()?;
    let tracks = serde_json::from_str::<Vec<AudioTrackCandidate>>(&request.tracks_json).ok()?;
    let passthrough = parse_string_list(&request.passthrough_track_ids_json);
    let sample_rates = parse_i32_list(&request.supported_sample_rates_json);
    let preferred = request.preferred_language.trim().to_lowercase();
    let candidates: Vec<AudioTrackCandidate> = tracks
        .into_iter()
        .filter(|track| track.is_supported)
        .collect();
    if candidates.is_empty() {
        return Some("null".to_string());
    }

    let language_matches = |track: &AudioTrackCandidate| {
        let language = track.language.as_deref().unwrap_or_default().to_lowercase();
        !preferred.is_empty()
            && (language == preferred
                || language.split('-').next() == preferred.split('-').next())
    };
    let pool: Vec<AudioTrackCandidate> = if candidates.iter().any(language_matches) {
        candidates.into_iter().filter(language_matches).collect()
    } else {
        candidates
    };

    let selected = pool.into_iter().max_by(|left, right| {
        audio_track_score(left, &passthrough, &sample_rates, request.max_pcm_channels)
            .cmp(&audio_track_score(
                right,
                &passthrough,
                &sample_rates,
                request.max_pcm_channels,
            ))
            .then_with(|| left.id.cmp(&right.id))
    })?;
    serde_json::to_string(&selected.id).ok()
}

fn audio_track_score(
    track: &AudioTrackCandidate,
    passthrough: &[String],
    sample_rates: &[i32],
    max_pcm_channels: Option<i32>,
) -> (i32, i32, i32, i32, i32, i64, i32) {
    let sample_rate_compatible = track
        .sample_rate
        .is_none_or(|rate| sample_rates.is_empty() || sample_rates.contains(&rate));
    let channels = track.channel_count.unwrap_or(2);
    (
        i32::from(passthrough.iter().any(|id| id == &track.id)),
        i32::from(sample_rate_compatible),
        codec_rank(track.sample_mime_type.as_deref()),
        i32::from(max_pcm_channels.is_none_or(|max| channels <= max)),
        channels,
        track.bitrate.unwrap_or(0),
        i32::from(track.is_selected),
    )
}

fn codec_rank(mime: Option<&str>) -> i32 {
    let value = mime.unwrap_or_default().to_lowercase().replace(['-', '.'], "");
    if ["truehd", "dtshd", "dtsx", "dtsuhd", "dolbymat", "mpegh", "dra"]
        .iter()
        .any(|codec| value.contains(codec))
    {
        600
    } else if [
        "flac", "alac", "pcm", "raw", "wav", "aiff", "ape", "monkeysaudio", "wavpack",
        "tta", "tak", "shn", "mlp",
    ]
    .iter()
    .any(|codec| value.contains(codec))
    {
        500
    } else if ["eac3", "ac3", "ac4", "dts"].iter().any(|codec| value.contains(codec)) {
        400
    } else if ["opus", "vorbis"].iter().any(|codec| value.contains(codec)) {
        300
    } else if ["aac", "mp4a"].iter().any(|codec| value.contains(codec)) {
        200
    } else if ["mpeg", "mp3"].iter().any(|codec| value.contains(codec)) {
        100
    } else {
        0
    }
}

fn parse_string_list(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn parse_i32_list(value: &str) -> Vec<i32> {
    serde_json::from_str::<Vec<i32>>(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::select_audio_track_json;
    use serde_json::Value;

    #[test]
    fn language_and_route_compatibility_precede_codec_quality() {
        let result: Value = serde_json::from_str(
            &select_audio_track_json(
                r#"{"tracksJson":"[{\"id\":\"en_truehd\",\"language\":\"en\",\"sampleMimeType\":\"audio/true-hd\",\"channelCount\":8},{\"id\":\"tr_eac3\",\"language\":\"tr\",\"sampleMimeType\":\"audio/eac3\",\"channelCount\":6}]","preferredLanguage":"tr","passthroughTrackIdsJson":"[]","supportedSampleRatesJson":"[]"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result, "tr_eac3");
    }

    #[test]
    fn passthrough_and_lossless_surround_are_ranked() {
        let result: Value = serde_json::from_str(
            &select_audio_track_json(
                r#"{"tracksJson":"[{\"id\":\"aac\",\"language\":\"en\",\"sampleMimeType\":\"audio/mp4a-latm\",\"channelCount\":2},{\"id\":\"flac\",\"language\":\"en\",\"sampleMimeType\":\"audio/flac\",\"channelCount\":6},{\"id\":\"eac3\",\"language\":\"en\",\"sampleMimeType\":\"audio/eac3\",\"channelCount\":6}]","preferredLanguage":"en","passthroughTrackIdsJson":"[\"eac3\"]","supportedSampleRatesJson":"[]","maxPcmChannels":2}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result, "eac3");
    }
}
