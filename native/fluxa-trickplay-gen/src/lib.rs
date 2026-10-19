//! Keyframe-only trickplay sheet generation.
//!
//! The generator never scans a stream end to end. Every supported container
//! exposes keyframes as self-contained bytes: HLS segments begin with one,
//! MP4 files index them in the moov atom, Matroska files list them in the
//! Cues element. One small range request per keyframe is enough to decode a
//! frame, which keeps preview traffic roughly two orders of magnitude below
//! a full-stream scan.
//!
//! The pipeline refines in strides: the first milestone covers the whole
//! timeline coarsely (every Nth keyframe) and later strides fill the gaps,
//! so the user sees a usable preview long before the final sheet set lands.

use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};

pub mod decode;
pub mod hls;
pub mod mkv;
pub mod mp4;
pub mod sample;
pub mod sheet;
#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Format(&'static str),
    Decode(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => write!(f, "http: {e}"),
            Error::Format(m) => write!(f, "container: {m}"),
            Error::Decode(m) => write!(f, "decode: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub use decode::{AnnexBFormat, FfmpegCliDecoder, FrameDecoder, FramePayload};
pub use sheet::{Sheet, SheetSpec};

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub headers: Vec<(String, String)>,
    pub strides: Vec<usize>,
    pub concurrency: usize,
    pub spec: SheetSpec,
    pub decoder: Arc<dyn FrameDecoder>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            strides: vec![6, 3, 1],
            concurrency: 8,
            spec: SheetSpec::default(),
            decoder: Arc::new(FfmpegCliDecoder::default()),
        }
    }
}

#[derive(Debug)]
pub enum TrickplayEvent {
    Milestone {
        frames: usize,
        grid_seconds: f64,
        sheets: Vec<Sheet>,
    },
    Done {
        frames: usize,
        duration: f64,
        sheets: Vec<Sheet>,
        vtt: String,
    },
}

/// One decodable keyframe and where its bytes live.
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f64,
    /// Byte offset inside the resource for indexed containers.
    pub offset: u64,
    /// Bytes to request.
    pub len: u32,
    /// For MP4 the raw sample is length-prefixed NAL units that must be
    /// rewritten to Annex B together with the codec description.
    pub annex_b: Option<AnnexBCall>,
}

#[derive(Debug, Clone)]
pub struct AnnexBCall {
    pub format: AnnexBFormat,
    /// Codec description (avcC / hvcC box payload).
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Hls,
    Mp4,
    Matroska,
}

pub struct KeyframeIndex {
    pub duration: f64,
    pub keyframes: Vec<Keyframe>,
    pub kind: SourceKind,
    /// Bytes prepended to Matroska frame payloads so a head slice plus one
    /// cluster parses as a standalone file.
    pub payload_prefix: Vec<u8>,
}

pub struct Generator {
    client: reqwest::Client,
    url: String,
    headers: Vec<(String, String)>,
}

impl Generator {
    pub fn new(url: &str, headers: Vec<(String, String)>) -> Result<Self> {
        let mut builder = reqwest::Client::builder();
        let mut default_ua = true;
        for (k, v) in &headers {
            if k.eq_ignore_ascii_case("user-agent") {
                default_ua = false;
            }
            builder = builder.default_headers(
                [(k.clone(), v.parse().map_err(|_| Error::Format("bad header value"))?)]
                    .into_iter()
                    .collect(),
            );
        }
        if default_ua {
            builder = builder.header(
                "user-agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
            );
        }
        Ok(Self {
            client: builder.build()?,
            url: url.to_owned(),
            headers,
        })
    }

    async fn request(&self, url: Option<&str>) -> reqwest::RequestBuilder {
        let mut req = self.client.get(url.unwrap_or(&self.url));
        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }
        req
    }

    async fn fetch_range(&self, offset: u64, len: u32) -> Result<Vec<u8>> {
        let resp = self
            .request(None)
            .await
            .header("range", format!("bytes={offset}-{}", offset + len as u64 - 1))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }

    async fn fetch_exact(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let mut got = self.fetch_range(offset, len.min(u32::MAX as u64) as u32).await?;
        while (got.len() as u64) < len {
            let more = self
                .fetch_range(offset + got.len() as u64, (len - got.len() as u64) as u32)
                .await?;
            if more.is_empty() {
                break;
            }
            got.extend_from_slice(&more);
        }
        got.truncate(len as usize);
        Ok(got)
    }

    async fn fetch_all(&self) -> Result<Vec<u8>> {
        Ok(self.request(None).await.send().await?.error_for_status()?.bytes().await?.to_vec())
    }

    async fn fetch_text(&self, url: &str) -> Result<String> {
        Ok(self.request(Some(url)).await.send().await?.error_for_status()?.text().await?)
    }

    async fn fetch_file_size(&self) -> Result<u64> {
        let resp = self
            .request(None)
            .await
            .header("range", "bytes=0-0")
            .send()
            .await?
            .error_for_status()?;
        if resp.status() == reqwest::StatusCode::OK {
            return Ok(resp.content_length().unwrap_or(0));
        }
        let range = resp
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.rsplit('/').next().map(|s| s.to_owned()));
        range
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(Error::Format("no content-range from server"))
    }

    pub async fn open_index(&self) -> Result<KeyframeIndex> {
        let head = self.fetch_range(0, 131_072).await?;
        if std::str::from_utf8(&head).map(|t| t.contains("#EXTM3U")).unwrap_or(false) {
            return self.open_hls().await;
        }
        match mp4::KeyframeTable::parse(&head, self).await {
            Ok(table) => return Ok(table),
            Err(mp4::Mp4Error::NotMp4) => {}
            Err(e) => return Err(Error::Format(e.message())),
        }
        match mkv::KeyframeTable::parse(&head, self).await {
            Ok(table) => Ok(table),
            Err(mkv::MkvError::NotMatroska) => Err(Error::Format(
                "not an HLS playlist, MP4 file, or Matroska file",
            )),
            Err(e) => Err(Error::Format(e.message())),
        }
    }

    async fn open_hls(&self) -> Result<KeyframeIndex> {
        let mut master = hls::parse(&self.fetch_all().await?).map_err(Error::Format)?;
        if let Some(variant) = master.lowest_variant_url.take() {
            let variant_url = hls::resolve(&self.url, &variant);
            let body = self.fetch_text(&variant_url).await?;
            master = hls::parse(body.as_bytes()).map_err(Error::Format)?;
        }
        let keyframes = master
            .segments
            .iter()
            .map(|s| Keyframe {
                time: s.start,
                offset: 0,
                len: 131_072,
                annex_b: None,
            })
            .collect();
        Ok(KeyframeIndex {
            duration: master.duration,
            keyframes,
            kind: SourceKind::Hls,
            payload_prefix: Vec::new(),
        })
    }

    async fn fetch_frame(&self, index: &KeyframeIndex, kf: &Keyframe) -> Result<FramePayload> {
        match index.kind {
            SourceKind::Hls => {
                let mut len = kf.len;
                let mut body = self.fetch_range(kf.offset, len).await?;
                while body.len() < 4096 && len < 1 << 21 {
                    len *= 2;
                    body = self.fetch_range(kf.offset, len).await?;
                }
                Ok(FramePayload::PartialContainer(body))
            }
            SourceKind::Mp4 => {
                let sample = self.fetch_exact(kf.offset, kf.len as u64).await?;
                let call = kf.annex_b.as_ref().ok_or(Error::Format("missing codec info"))?;
                Ok(FramePayload::AnnexB {
                    format: call.format,
                    data: sample::to_annex_b(&sample, &call.data, call.format),
                })
            }
            SourceKind::Matroska => {
                let mut len = kf.len;
                let mut body = self.fetch_range(kf.offset, len).await?;
                while body.len() < 4096 && len < 1 << 21 {
                    len *= 2;
                    body = self.fetch_range(kf.offset, len).await?;
                }
                let mut payload = index.payload_prefix.clone();
                payload.extend_from_slice(&body);
                Ok(FramePayload::PartialContainer(payload))
            }
        }
    }

    pub async fn generate(
        &self,
        opts: GenerateOptions,
        events: mpsc::Sender<TrickplayEvent>,
    ) -> Result<()> {
        let index = self.open_index().await?;
        let total = index.keyframes.len();
        let mut done: Vec<Option<(f64, Vec<u8>)>> = (0..total).map(|_| None).collect();

        for (stride_idx, &stride) in opts.strides.iter().enumerate() {
            let last = stride_idx + 1 == opts.strides.len();
            let semaphore = Arc::new(Semaphore::new(opts.concurrency));
            let mut tasks = Vec::new();
            for i in 0..total {
                if done[i].is_some() || (!last && i % stride != 0) {
                    continue;
                }
                let permit = semaphore.clone().acquire_owned().await;
                let kf = index.keyframes[i].clone();
                let payload = self.fetch_frame(&index, &kf).await;
                let decoder = opts.decoder.clone();
                tasks.push(tokio::spawn(async move {
                    let _permit = permit;
                    let jpeg = match payload {
                        Ok(p) => decoder.decode_first_keyframe(p).await,
                        Err(_) => None,
                    };
                    (i, jpeg)
                }));
            }
            for task in tasks {
                let (i, jpeg) = task.await.map_err(|_| Error::Decode("worker panicked".into()))?;
                if let Some(jpeg) = jpeg {
                    done[i] = Some((index.keyframes[i].time, jpeg));
                }
            }
            if !last {
                let frames: Vec<(f64, Vec<u8>)> =
                    done.iter().filter_map(|s| s.as_ref().cloned()).collect();
                let sheets = sheet::tile(&frames, &opts.spec);
                if !sheets.is_empty() {
                    let _ = events
                        .send(TrickplayEvent::Milestone {
                            frames: frames.len(),
                            grid_seconds: index.duration / frames.len().max(1) as f64,
                            sheets,
                        })
                        .await;
                }
            }
        }

        let frames: Vec<(f64, Vec<u8>)> = done.iter().filter_map(|s| s.as_ref().cloned()).collect();
        let times: Vec<f64> = frames.iter().map(|(t, _)| *t).collect();
        let sheets = sheet::tile(&frames, &opts.spec);
        let vtt = sheet::vtt(&times, &opts.spec, "{sheet}");
        let _ = events
            .send(TrickplayEvent::Done {
                frames: frames.len(),
                duration: index.duration,
                sheets,
                vtt,
            })
            .await;
        Ok(())
    }
}
