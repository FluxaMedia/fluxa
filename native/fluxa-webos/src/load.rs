use serde_json::{Value, json};

use crate::base64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VideoCodec {
    H264,
    H265,
    Vp8,
    Vp9,
    Av1,
}

impl VideoCodec {
    fn name(self) -> &'static str {
        match self {
            VideoCodec::H264 => "H264",
            VideoCodec::H265 => "H265",
            VideoCodec::Vp8 => "VP8",
            VideoCodec::Vp9 => "VP9",
            VideoCodec::Av1 => "AV1",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AudioCodec {
    Aac { profile: u8, raw: bool },
    Ac3,
    Eac3 { atmos: bool },
    Ac4,
    Dts,
    DtsEs,
    DtsX,
    Opus,
    Mp3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DolbyVision {
    pub profile: u8,
    pub dual_layer: bool,
}

#[derive(Clone, Debug)]
pub struct VideoStream {
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub fps: Option<(u32, u32)>,
    pub dolby_vision: Option<DolbyVision>,
}

#[derive(Clone, Debug)]
pub struct AudioStream {
    pub codec: AudioCodec,
    pub channels: u32,
    pub sample_rate: u32,
    pub extradata: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct LoadOptions {
    pub app_id: String,
    pub webos_major: u32,
    pub start_pts_ns: i64,
    pub window_id: Option<String>,
    pub max_resolution: Option<(u32, u32, u32)>,
}

const MAX_SRC_BUFFER_AUDIO: u64 = 2 * 1024 * 1024;
const MAX_SRC_BUFFER_VIDEO: u64 = 8 * 1024 * 1024;

fn audio_info(audio: &AudioStream, contents: &mut Value) -> &'static str {
    let freq = audio.sample_rate as f64 / 1000.0;
    let info = |channels: u32| json!({ "channels": channels, "frequency": freq });
    match audio.codec {
        AudioCodec::Aac { profile, raw } => {
            contents["aacInfo"] = json!({
                "channels": audio.channels,
                "profile": profile as u32 + 1,
                "format": if raw { "raw" } else { "adts" },
                "frequency": freq,
            });
            "AAC"
        }
        AudioCodec::Ac3 => "AC3",
        AudioCodec::Eac3 { atmos } => {
            contents["ac3PlusInfo"] = info(audio.channels + if atmos { 2 } else { 0 });
            if atmos {
                contents["immersive"] = json!("ATMOS");
            }
            "AC3 PLUS"
        }
        AudioCodec::Ac4 => {
            contents["ac4Info"] = info(audio.channels);
            "AC4"
        }
        AudioCodec::Dts | AudioCodec::DtsEs | AudioCodec::DtsX => {
            contents["dtsInfo"] = info(audio.channels);
            match audio.codec {
                AudioCodec::DtsEs => "DTSE",
                AudioCodec::DtsX => "DTSX",
                _ => "DTS",
            }
        }
        AudioCodec::Opus => {
            contents["opusInfo"] = json!({
                "channels": audio.channels,
                "frequency": freq,
                "streamHeader": base64::encode(&audio.extradata),
            });
            "OPUS"
        }
        AudioCodec::Mp3 => "MP3",
    }
}

pub fn load_payload(video: &VideoStream, audio: Option<&AudioStream>, options: &LoadOptions) -> String {
    let mut contents = json!({
        "codec": { "video": video.codec.name() },
        "format": "RAW",
        "provider": options.app_id,
    });
    if let Some(dv) = video.dolby_vision {
        contents["DolbyHdrInfo"] = json!({
            "encryptionType": "clear",
            "profileId": dv.profile,
            "trackType": if dv.dual_layer { "dual" } else { "single" },
        });
    }

    let mut es_info = json!({
        "pauseAtDecodeTime": true,
        "seperatedPTS": true,
        "ptsToDecode": options.start_pts_ns,
        "videoWidth": video.width,
        "videoHeight": video.height,
    });
    if let Some((value, scale)) = video.fps.filter(|(value, scale)| *value > 0 && *scale > 0) {
        es_info["videoFpsValue"] = json!(value);
        es_info["videoFpsScale"] = json!(scale);
    }
    contents["esInfo"] = es_info;

    let mut option = json!({
        "appId": options.app_id,
        "transmission": { "contentsType": "LIVE", "trickType": "client-side" },
        "seekMode": "keep-rate",
        "useDroppedFrameEvent": true,
    });

    match audio {
        Some(audio) => {
            let name = audio_info(audio, &mut contents);
            contents["codec"]["audio"] = json!(name);
        }
        None => option["needAudio"] = json!(false),
    }

    option["externalStreamingInfo"] = json!({
        "contents": contents,
        "streamQualityInfo": true,
        "streamQualityInfoNonFlushable": true,
        "streamQualityInfoCorruptedFrame": true,
        "bufferingCtrInfo": {
            "preBufferByte": 0,
            "bufferMinLevel": 0,
            "bufferMaxLevel": 0,
            "qBufferLevelVideo": 0,
            "srcBufferLevelVideo": { "minimum": 0, "maximum": MAX_SRC_BUFFER_VIDEO },
            "qBufferLevelAudio": 0,
            "srcBufferLevelAudio": { "minimum": 0, "maximum": MAX_SRC_BUFFER_AUDIO },
        },
    });

    if let Some(window) = &options.window_id {
        option["windowId"] = json!(window);
    }
    if let Some((width, height, frame_rate)) = options.max_resolution {
        option["adaptiveStreaming"] = json!({
            "adaptiveResolution": true,
            "maxWidth": width,
            "maxHeight": height,
            "maxFrameRate": frame_rate,
        });
    }

    json!({ "args": [{ "mediaTransportType": "BUFFERSTREAM", "option": option }] }).to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn options() -> LoadOptions {
        LoadOptions {
            app_id: "com.fluxa.app".into(),
            webos_major: 5,
            start_pts_ns: 0,
            window_id: None,
            max_resolution: None,
        }
    }

    fn video() -> VideoStream {
        VideoStream {
            codec: VideoCodec::H265,
            width: 3840,
            height: 2160,
            fps: Some((24000, 1001)),
            dolby_vision: None,
        }
    }

    fn parse(payload: &str) -> Value {
        serde_json::from_str(payload).unwrap()
    }

    #[test]
    fn missing_audio_sets_need_audio_false() {
        let json = parse(&load_payload(&video(), None, &options()));
        let option = &json["args"][0]["option"];
        assert_eq!(option["needAudio"], false);
        assert!(option["externalStreamingInfo"]["contents"]["codec"].get("audio").is_none());
    }

    #[test]
    fn atmos_adds_two_channels_and_immersive_flag() {
        let audio = AudioStream {
            codec: AudioCodec::Eac3 { atmos: true },
            channels: 6,
            sample_rate: 48000,
            extradata: vec![],
        };
        let json = parse(&load_payload(&video(), Some(&audio), &options()));
        let contents = &json["args"][0]["option"]["externalStreamingInfo"]["contents"];
        assert_eq!(contents["codec"]["audio"], "AC3 PLUS");
        assert_eq!(contents["ac3PlusInfo"]["channels"], 8);
        assert_eq!(contents["immersive"], "ATMOS");
    }

    #[test]
    fn dolby_vision_single_layer_declares_profile() {
        let mut stream = video();
        stream.dolby_vision = Some(DolbyVision { profile: 8, dual_layer: false });
        let json = parse(&load_payload(&stream, None, &options()));
        let dv = &json["args"][0]["option"]["externalStreamingInfo"]["contents"]["DolbyHdrInfo"];
        assert_eq!(dv["profileId"], 8);
        assert_eq!(dv["trackType"], "single");
    }

    #[test]
    fn opus_carries_base64_header() {
        let audio = AudioStream {
            codec: AudioCodec::Opus,
            channels: 2,
            sample_rate: 48000,
            extradata: b"foo".to_vec(),
        };
        let json = parse(&load_payload(&video(), Some(&audio), &options()));
        let contents = &json["args"][0]["option"]["externalStreamingInfo"]["contents"];
        assert_eq!(contents["opusInfo"]["streamHeader"], "Zm9v");
    }

    #[test]
    fn window_id_only_present_when_exported() {
        let mut with_window = options();
        with_window.window_id = Some("_Window_Id_1".into());
        let json = parse(&load_payload(&video(), None, &with_window));
        assert_eq!(json["args"][0]["option"]["windowId"], "_Window_Id_1");
        let json = parse(&load_payload(&video(), None, &options()));
        assert!(json["args"][0]["option"].get("windowId").is_none());
    }
}
