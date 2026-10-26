//! Frame decoding seam. The default implementation shells out to an ffmpeg
//! binary; the trait exists so a host can swap in libavcodec-linked decoding
//! without touching the pipeline.

use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnexBFormat {
    H264,
    Hevc,
}

/// Bytes handed to the decoder for exactly one keyframe.
#[derive(Debug)]
pub enum FramePayload {
    /// A head slice of an HLS segment or a Matroska header+cluster blob;
    /// decoders parse the container themselves.
    PartialContainer(Vec<u8>),
    /// Annex B stream (start codes, SPS/PPS prepended) holding one keyframe.
    AnnexB { format: AnnexBFormat, data: Vec<u8> },
}

pub trait FrameDecoder: Send + Sync {
    /// Returns JPEG bytes for the first keyframe in the payload.
    fn decode_first_keyframe(
        &self,
        payload: FramePayload,
    ) -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send + '_>>;
}

pub struct FfmpegCliDecoder {
    pub binary: String,
    pub timeout: std::time::Duration,
}

impl Default for FfmpegCliDecoder {
    fn default() -> Self {
        Self {
            binary: "ffmpeg".to_owned(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl FrameDecoder for FfmpegCliDecoder {
    fn decode_first_keyframe(
        &self,
        payload: FramePayload,
    ) -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send + '_>> {
        let binary = self.binary.clone();
        let timeout = self.timeout;
        Box::pin(async move {
            let (mut cmd, stdin_data) = match payload {
                FramePayload::PartialContainer(data) => {
                    let mut cmd = tokio::process::Command::new(&binary);
                    cmd.args([
                        "-v",
                        "error",
                        "-i",
                        "pipe:0",
                        "-vf",
                        "select='eq(pict_type,I)'",
                        "-frames:v",
                        "1",
                        "-f",
                        "image2pipe",
                        "-c:v",
                        "mjpeg",
                        "pipe:1",
                    ]);
                    (cmd, data)
                }
                FramePayload::AnnexB { format, data } => {
                    let mut cmd = tokio::process::Command::new(&binary);
                    let fmt = match format {
                        AnnexBFormat::H264 => "h264",
                        AnnexBFormat::Hevc => "hevc",
                    };
                    cmd.args([
                        "-v",
                        "error",
                        "-f",
                        fmt,
                        "-i",
                        "pipe:0",
                        "-frames:v",
                        "1",
                        "-f",
                        "image2pipe",
                        "-c:v",
                        "mjpeg",
                        "pipe:1",
                    ]);
                    (cmd, data)
                }
            };
            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null());
            let mut child = cmd.spawn().ok()?;
            use tokio::io::AsyncWriteExt;
            let mut stdin = child.stdin.take()?;
            let _ = stdin.write_all(&stdin_data).await;
            drop(stdin);
            let out = tokio::time::timeout(timeout, child.wait_with_output())
                .await
                .ok()?
                .ok()?;
            if out.stdout.is_empty() {
                None
            } else {
                Some(out.stdout)
            }
        })
    }
}
