use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioPcmChannelsRequest {
    #[serde(default)]
    device_max_channels: Option<i64>,
    #[serde(default)]
    capabilities_max_channels: i64,
    #[serde(default)]
    speaker_layout_max_channels: Option<i64>,
    #[serde(default)]
    spatializer_max_channels: Option<i64>,
    #[serde(default)]
    route_supports_multichannel: bool,
}

pub(crate) fn audio_pcm_channel_count_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<AudioPcmChannelsRequest>(request_json)
        .map_err(|error| CoreError::BadInput {
            context: "audio_pcm_channel_count_json",
            detail: error.to_string(),
        })
        .log_discard()?;
    let speaker_layout_max = request.speaker_layout_max_channels.unwrap_or_default();
    let spatializer_max = request.spatializer_max_channels.unwrap_or_default();
    let route_pcm_max = request
        .capabilities_max_channels
        .max(speaker_layout_max.max(spatializer_max))
        .clamp(2, 8);
    let result = match request.device_max_channels {
        Some(device_max) => device_max
            .max(if request.route_supports_multichannel {
                speaker_layout_max
            } else {
                0
            })
            .max(if request.route_supports_multichannel {
                spatializer_max
            } else {
                0
            })
            .min(route_pcm_max)
            .clamp(2, 8),
        None if request.route_supports_multichannel => request
            .capabilities_max_channels
            .max(speaker_layout_max)
            .max(spatializer_max)
            .clamp(2, 8),
        None if request.spatializer_max_channels.is_some() => spatializer_max
            .min(request.capabilities_max_channels)
            .clamp(2, 8),
        None => 2,
    };
    Some(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::audio_pcm_channel_count_json;

    #[test]
    fn keeps_multichannel_layout_when_hdmi_endpoint_reports_stereo() {
        assert_eq!(
            audio_pcm_channel_count_json(
                r#"{"deviceMaxChannels":2,"capabilitiesMaxChannels":2,"speakerLayoutMaxChannels":8,"routeSupportsMultichannel":true}"#
            ),
            Some("8".to_string())
        );
    }

    #[test]
    fn remains_conservative_for_stereo_headphones() {
        assert_eq!(
            audio_pcm_channel_count_json(
                r#"{"deviceMaxChannels":2,"capabilitiesMaxChannels":8,"speakerLayoutMaxChannels":8,"routeSupportsMultichannel":false}"#
            ),
            Some("2".to_string())
        );
    }

    #[test]
    fn spatializer_can_advertise_multichannel_input_without_device_channel_list() {
        assert_eq!(
            audio_pcm_channel_count_json(
                r#"{"capabilitiesMaxChannels":8,"spatializerMaxChannels":6,"routeSupportsMultichannel":false}"#
            ),
            Some("6".to_string())
        );
    }
}
