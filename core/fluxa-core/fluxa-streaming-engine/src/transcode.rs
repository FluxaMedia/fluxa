use axum::Router;
use axum::body::Body;
use axum::extract::Query;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use bytes::Bytes;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::process::{Child, ChildStdout, Command};
use tokio_util::io::ReaderStream;

use crate::ffmpeg_locator;

/// Kills and reaps the ffmpeg child when dropped — whether that's because
/// the response stream finished normally (process already exited, so the
/// kill is a harmless no-op) or because the client disconnected mid-stream,
/// in which case ffmpeg would otherwise just stall on a full stdout pipe
/// instead of actually exiting.
struct KillOnDrop(Option<Child>);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.start_kill();
            tokio::spawn(async move {
                let _ = child.wait().await;
            });
        }
    }
}

struct ChildStdoutGuarded {
    stdout: ChildStdout,
    _guard: KillOnDrop,
}

impl AsyncRead for ChildStdoutGuarded {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stdout).poll_read(cx, buf)
    }
}

#[derive(Deserialize)]
pub struct TranscodeQuery {
    url: String,
    /// Input-side seek in seconds. The transcode itself isn't byte-range
    /// seekable (it's a live ffmpeg pipe), so the player seeks by re-opening
    /// this endpoint with a new `start` instead.
    start: Option<f64>,
    #[serde(rename = "hwEncoder")]
    hw_encoder: Option<String>,
    #[serde(rename = "streamBufferBytes")]
    stream_buffer_bytes: Option<usize>,
    #[serde(rename = "videoCodec")]
    video_codec: Option<String>,
    #[serde(rename = "audioCodec")]
    audio_codec: Option<String>,
    container: Option<String>,
}

/// Blocks ffmpeg url schemes like `file:` and SSRF to non-loopback hosts.
fn is_allowed_stream_url(raw: &str) -> bool {
    let Ok(parsed) = url::Url::parse(raw) else {
        return false;
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    matches!(
        parsed.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("[::1]") | Some("::1")
    )
}

#[derive(Clone, Default)]
struct ProbedCodecs {
    video: Option<String>,
    audio: Option<String>,
    audio_channels: Option<u32>,
    duration: Option<f64>,
}

async fn probe(url: &str) -> ProbedCodecs {
    static CACHE: OnceLock<Mutex<HashMap<String, ProbedCodecs>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(codecs) = cache.lock().ok().and_then(|cache| cache.get(url).cloned()) {
        return codecs;
    }

    let ffprobe = ffmpeg_locator::resolve("ffprobe");
    let output = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_entries",
            "stream=codec_type,codec_name,channels:format=duration",
        ])
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .await;

    let Ok(output) = output else {
        return ProbedCodecs::default();
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return ProbedCodecs::default();
    };
    let mut codecs = ProbedCodecs::default();
    if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
        let kind = stream.get("codec_type").and_then(|v| v.as_str());
        let name = stream
                .get("codec_name")
                .and_then(|v| v.as_str())
                .map(str::to_string);
        match kind {
            Some("video") if codecs.video.is_none() => codecs.video = name,
            Some("audio") if codecs.audio.is_none() => {
                codecs.audio = name;
                codecs.audio_channels = stream
                    .get("channels")
                    .and_then(|value| value.as_u64())
                    .and_then(|value| u32::try_from(value).ok());
            }
            _ => {}
        }
        }
    }
    codecs.duration = json
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse::<f64>().ok());
    if let Ok(mut cache) = cache.lock() {
        if cache.len() >= 64 {
            cache.clear();
        }
        cache.insert(url.to_string(), codecs.clone());
    }
    codecs
}

#[derive(Deserialize)]
pub struct ProbeQuery {
    url: String,
}

pub async fn handle_probe(Query(q): Query<ProbeQuery>) -> Response {
    if !is_allowed_stream_url(&q.url) {
        return (
            StatusCode::BAD_REQUEST,
            "url must be http(s)://127.0.0.1 or localhost",
        )
            .into_response();
    }
    let codecs = probe(&q.url).await;
    axum::Json(serde_json::json!({
        "videoCodec": codecs.video,
        "audioCodec": codecs.audio,
        "duration": codecs.duration,
    }))
    .into_response()
}

/// Remuxes (stream-copy) when the source codecs are already browser-playable,
/// and falls back to a real transcode only for the tracks that aren't —
/// most addon releases are h264+aac in an mkv container, so this is a cheap
/// container rewrite rather than a full re-encode in the common case.
pub async fn handle_transcode(Query(q): Query<TranscodeQuery>) -> Response {
    if !is_allowed_stream_url(&q.url) {
        return (
            StatusCode::BAD_REQUEST,
            "url must be http(s)://127.0.0.1 or localhost",
        )
            .into_response();
    }
    let codecs = probe(&q.url).await;
    let hw_encoder = resolve_hw_encoder(q.hw_encoder.as_deref()).await;

    let requested_video = normalized_codec(q.video_codec.as_deref()).or_else(|| codecs.video.clone().filter(|codec| codec == "h264"));
    let video_codec = requested_video.as_deref().unwrap_or("h264");
    let video_args = if codecs.video.as_deref() == Some(video_codec) {
        vec!["-c:v", "copy"]
    } else if video_codec == "h264" {
        hardware_video_args(hw_encoder)
            .unwrap_or_else(|| vec![
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "18",
                "-pix_fmt",
                "yuv420p",
            ])
    } else if video_codec == "vp9" {
        vec!["-c:v", "libvpx-vp9", "-crf", "30", "-b:v", "0"]
    } else if video_codec == "vp8" {
        vec!["-c:v", "libvpx", "-crf", "10", "-b:v", "0"]
    } else if video_codec == "av1" {
        vec!["-c:v", "libaom-av1", "-crf", "30", "-b:v", "0"]
    } else {
        vec!["-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p"]
    };
    let audio_bitrate = match codecs.audio_channels.unwrap_or(2) {
        1..=2 => "320k",
        3..=6 => "448k",
        _ => "512k",
    };
    let requested_audio = normalized_codec(q.audio_codec.as_deref()).or_else(|| codecs.audio.clone().filter(|codec| codec == "aac"));
    let audio_codec = requested_audio.as_deref().unwrap_or("aac");
    let audio_args: Vec<&str> = match codecs.audio.as_deref() {
        None => vec!["-an"],
        Some(source) if source == audio_codec => vec!["-c:a", "copy"],
        Some(_) if audio_codec == "opus" => vec!["-c:a", "libopus", "-b:a", "256k"],
        Some(_) if audio_codec == "vorbis" => vec!["-c:a", "libvorbis", "-q:a", "8"],
        Some(_) if audio_codec == "mp3" => vec!["-c:a", "libmp3lame", "-b:a", audio_bitrate],
        Some(_) => vec!["-c:a", "aac", "-b:a", audio_bitrate],
    };
    let container = match q.container.as_deref() {
        Some("webm")
            if (video_codec == "vp8" || video_codec == "vp9" || video_codec == "av1")
                && (codecs.audio.is_none() || audio_codec == "opus" || audio_codec == "vorbis") =>
        {
            "webm"
        }
        _ => "mp4",
    };

    let ffmpeg = ffmpeg_locator::resolve("ffmpeg");
    let mut cmd = Command::new(ffmpeg);
    if let Some(start) = q.start.filter(|s| *s > 0.0) {
        cmd.args(["-ss", &start.to_string()]);
    }
    cmd.arg("-i")
        .arg(&q.url)
        .args(["-map", "0:v:0"])
        .args(["-map", "0:a:0?"])
        .args(video_args)
        .args(audio_args)
        .args(["-map_metadata", "0", "-sn", "-avoid_negative_ts", "make_zero"])
        .args(if container == "mp4" { vec!["-movflags", "frag_keyframe+empty_moov+default_base_moof"] } else { Vec::new() })
        .args(["-f", container, "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to start ffmpeg: {e}"),
            )
                .into_response();
        }
    };
    let Some(stdout) = child.stdout.take() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ffmpeg produced no stdout pipe",
        )
            .into_response();
    };

    // Tied to the response body's lifetime via KillOnDrop, so a client
    // disconnect kills ffmpeg immediately instead of leaving it stalled on
    // a stdout pipe nobody's reading from.
    let guarded = ChildStdoutGuarded {
        stdout,
        _guard: KillOnDrop(Some(child)),
    };
    let buffer_size = stream_buffer_size(q.stream_buffer_bytes);
    let body = Body::from_stream(ReaderStream::with_capacity(guarded, buffer_size));
    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(if container == "webm" { "video/webm" } else { "video/mp4" }),
    );
    response
}

/// Streams a supported Matroska source as rolling fragmented MP4 without
/// decoding or re-encoding its video. The input is normally the companion's
/// loopback `/proxy` URL, so the endpoint can stay behind the same SSRF gate
/// as `/transcode`.
pub async fn handle_remux(Query(q): Query<TranscodeQuery>) -> Response {
    if !is_allowed_stream_url(&q.url) {
        return (
            StatusCode::BAD_REQUEST,
            "url must be http(s)://127.0.0.1 or localhost",
        )
            .into_response();
    }

    let response = match reqwest::Client::new().get(&q.url).send().await {
        Ok(response) if response.status().is_success() => response,
        Ok(response) => {
            return (response.status(), "upstream source was not successful").into_response();
        }
        Err(error) => return (StatusCode::BAD_GATEWAY, error.to_string()).into_response(),
    };

    let input = response.bytes_stream();
    let output = futures_util::stream::unfold(
        (input, fluxa_core::media_demux::IncrementalFmp4Session::new(), false),
        |(mut input, mut session, mut finished)| async move {
            loop {
                if finished {
                    return None;
                }
                if let Some(chunk) = input.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(error) => {
                            finished = true;
                            return Some((
                                Err(std::io::Error::other(error.to_string())),
                                (input, session, finished),
                            ));
                        }
                    };
                    let bytes = session.push(&chunk);
                    if !bytes.is_empty() {
                        return Some((Ok(Bytes::from(bytes)), (input, session, finished)));
                    }
                } else {
                    finished = true;
                    let bytes = session.finish();
                    if !bytes.is_empty() {
                        return Some((Ok(Bytes::from(bytes)), (input, session, finished)));
                    }
                    if session.is_supported() == Some(false) {
                        return Some((
                            Err(std::io::Error::other(
                                "source video codec cannot be remuxed to fMP4",
                            )),
                            (input, session, finished),
                        ));
                    }
                    return None;
                }
            }
        },
    );
    let mut output = Body::from_stream(output).into_response();
    output.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("video/mp4"),
    );
    output
}

fn normalized_codec(codec: Option<&str>) -> Option<String> {
    let codec = codec?.to_ascii_lowercase();
    let normalized = if codec.contains("h264") || codec.contains("avc") {
        "h264"
    } else if codec.contains("hevc") || codec.contains("h265") {
        "hevc"
    } else if codec.contains("vp9") {
        "vp9"
    } else if codec.contains("vp8") {
        "vp8"
    } else if codec.contains("av1") {
        "av1"
    } else if codec.contains("aac") || codec.contains("mp4a") {
        "aac"
    } else if codec.contains("opus") {
        "opus"
    } else if codec.contains("vorbis") {
        "vorbis"
    } else if codec.contains("mp3") || codec.contains("mpeg") {
        "mp3"
    } else {
        return None;
    };
    Some(normalized.to_string())
}

fn stream_buffer_size(requested: Option<usize>) -> usize {
    requested.unwrap_or(64 * 1024).clamp(64 * 1024, 256 * 1024)
}

fn hardware_video_args(encoder: Option<&str>) -> Option<Vec<&'static str>> {
    match encoder {
        Some("nvenc") => Some(vec!["-c:v", "h264_nvenc", "-preset", "p4", "-cq", "20"]),
        Some("qsv") => Some(vec!["-c:v", "h264_qsv", "-global_quality", "23"]),
        Some("videotoolbox") => Some(vec!["-c:v", "h264_videotoolbox", "-q:v", "65"]),
        Some("vaapi") => Some(vec![
            "-vf",
            "format=nv12,hwupload",
            "-c:v",
            "h264_vaapi",
            "-qp",
            "20",
        ]),
        _ => None,
    }
}

async fn resolve_hw_encoder(requested: Option<&str>) -> Option<&str> {
    if requested != Some("auto") {
        let encoder = requested?;
        return probe_hardware_encoder(encoder).await.then_some(encoder);
    }
    static ENCODER: OnceLock<tokio::sync::OnceCell<Option<&'static str>>> = OnceLock::new();
    let cell = ENCODER.get_or_init(tokio::sync::OnceCell::new);
    cell.get_or_init(|| async {
        let output = Command::new(ffmpeg_locator::resolve("ffmpeg"))
            .args(["-hide_banner", "-encoders"])
            .stdin(Stdio::null())
            .output()
            .await
            .ok()?;
        let encoders = output.stdout;
        let candidates: &[(&str, &str)] = if cfg!(target_os = "macos") {
            &[("videotoolbox", "h264_videotoolbox"), ("qsv", "h264_qsv")]
        } else if cfg!(target_os = "windows") {
            &[("nvenc", "h264_nvenc"), ("qsv", "h264_qsv")]
        } else {
            &[
                ("nvenc", "h264_nvenc"),
                ("qsv", "h264_qsv"),
                ("vaapi", "h264_vaapi"),
            ]
        };
        for (name, encoder) in candidates {
            if !encoders
                .windows(encoder.len())
                .any(|window| window == encoder.as_bytes())
            {
                continue;
            }
            if probe_hardware_encoder(name).await {
                return Some(*name);
            }
        }
        None
    })
    .await
    .as_deref()
}

async fn probe_hardware_encoder(encoder: &str) -> bool {
    let Some(args) = hardware_video_args(Some(encoder)) else {
        return false;
    };
    Command::new(ffmpeg_locator::resolve("ffmpeg"))
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=16x16:rate=1",
            "-frames:v",
            "1",
        ])
        .args(args)
        .args(["-an", "-f", "null", "-"])
        .stdin(Stdio::null())
        .output()
        .await
        .is_ok_and(|output| output.status.success())
}

pub fn router() -> Router {
    Router::new()
        .route("/transcode", get(handle_transcode))
        .route("/remux", get(handle_remux))
        .route("/probe", get(handle_probe))
}

#[cfg(test)]
mod tests {
    use super::{hardware_video_args, is_allowed_stream_url, normalized_codec, stream_buffer_size};

    #[test]
    fn hardware_encoder_args_are_complete() {
        assert_eq!(
            hardware_video_args(Some("nvenc")),
            Some(vec!["-c:v", "h264_nvenc", "-preset", "p4", "-cq", "20"])
        );
        assert_eq!(
            hardware_video_args(Some("vaapi")),
            Some(vec![
                "-vf",
                "format=nv12,hwupload",
                "-c:v",
                "h264_vaapi",
                "-qp",
                "20"
            ])
        );
        assert!(hardware_video_args(Some("unknown")).is_none());
    }

    #[test]
    fn transcode_accepts_only_loopback_http_urls() {
        assert!(is_allowed_stream_url("http://127.0.0.1:8080/stream/x"));
        assert!(is_allowed_stream_url("https://localhost/stream/x"));
        assert!(!is_allowed_stream_url("file:///tmp/video.mkv"));
        assert!(!is_allowed_stream_url("https://example.com/video.mkv"));
    }

    #[test]
    fn stream_buffer_size_stays_within_device_budget() {
        assert_eq!(stream_buffer_size(None), 64 * 1024);
        assert_eq!(stream_buffer_size(Some(16 * 1024)), 64 * 1024);
        assert_eq!(stream_buffer_size(Some(512 * 1024)), 256 * 1024);
        assert_eq!(stream_buffer_size(Some(128 * 1024)), 128 * 1024);
    }

    #[test]
    fn software_video_fallback_uses_quality_first_browser_safe_settings() {
        let args = [
            "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
        ];
        assert!(args.windows(2).any(|pair| pair == ["-preset", "medium"]));
        assert!(args.windows(2).any(|pair| pair == ["-crf", "18"]));
        assert!(args.windows(2).any(|pair| pair == ["-pix_fmt", "yuv420p"]));
    }

    #[test]
    fn requested_codecs_are_normalized_to_safe_encoder_names() {
        assert_eq!(normalized_codec(Some("avc1.640028")), Some("h264".to_string()));
        assert_eq!(normalized_codec(Some("hevc")), Some("hevc".to_string()));
        assert_eq!(normalized_codec(Some("opus")), Some("opus".to_string()));
        assert_eq!(normalized_codec(Some("unsupported")), None);
    }
}
