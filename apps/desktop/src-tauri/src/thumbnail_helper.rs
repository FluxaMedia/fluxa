use crate::mpv_render::MpvThumbnailRenderer;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Deserialize, Serialize)]
struct ThumbnailRequest {
    url: String,
    time_pos: f64,
    #[serde(default)]
    prepare: bool,
}

#[derive(Deserialize, Serialize)]
struct ThumbnailResponse {
    ok: bool,
    image: Option<String>,
    error: Option<String>,
}

pub struct ThumbnailProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ThumbnailProcess {
    pub fn spawn() -> Result<Self, String> {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut child = Command::new(executable)
            .arg("--fluxa-thumbnail-helper")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("failed to start thumbnail helper: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "thumbnail helper stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "thumbnail helper stdout unavailable".to_string())?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn request(&mut self, url: &str, time_pos: f64) -> Result<String, String> {
        let started_at = std::time::Instant::now();
        let request = serde_json::to_string(&ThumbnailRequest {
            url: url.to_string(),
            time_pos,
            prepare: false,
        })
        .map_err(|error| error.to_string())?;
        writeln!(self.stdin, "{request}").map_err(|error| error.to_string())?;
        self.stdin.flush().map_err(|error| error.to_string())?;

        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if line.is_empty() {
            return Err("thumbnail helper exited unexpectedly".to_string());
        }
        let response: ThumbnailResponse =
            serde_json::from_str(&line).map_err(|error| error.to_string())?;
        eprintln!(
            "thumbnail_helper request time={time_pos:.3} elapsed_ms={}",
            started_at.elapsed().as_millis()
        );
        if response.ok {
            response
                .image
                .ok_or_else(|| "thumbnail helper returned no image".to_string())
        } else {
            Err(response
                .error
                .unwrap_or_else(|| "thumbnail failed".to_string()))
        }
    }

    pub fn prepare(&mut self, url: &str) -> Result<(), String> {
        let request = serde_json::to_string(&ThumbnailRequest {
            url: url.to_string(),
            time_pos: 0.0,
            prepare: true,
        })
        .map_err(|error| error.to_string())?;
        writeln!(self.stdin, "{request}").map_err(|error| error.to_string())?;
        self.stdin.flush().map_err(|error| error.to_string())?;
        let mut response = String::new();
        self.stdout
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

impl Drop for ThumbnailProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn run() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let mut renderer: Option<MpvThumbnailRenderer> = None;
    let mut loaded_url: Option<String> = None;

    for line in stdin.lock().lines() {
        let response = match line {
            Ok(line) => match serde_json::from_str::<ThumbnailRequest>(&line) {
                Ok(request) => match if request.prepare {
                    prepare_request(&mut renderer, &mut loaded_url, &request.url)
                } else {
                    render_request(
                        &mut renderer,
                        &mut loaded_url,
                        &request.url,
                        request.time_pos,
                    )
                } {
                    Ok(image) => ThumbnailResponse {
                        ok: true,
                        image: Some(image),
                        error: None,
                    },
                    Err(error) => ThumbnailResponse {
                        ok: false,
                        image: None,
                        error: Some(error),
                    },
                },
                Err(error) => ThumbnailResponse {
                    ok: false,
                    image: None,
                    error: Some(format!("invalid thumbnail request: {error}")),
                },
            },
            Err(error) => return Err(error.to_string()),
        };

        serde_json::to_writer(&mut stdout, &response).map_err(|error| error.to_string())?;
        stdout.write_all(b"\n").map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn prepare_request(
    renderer: &mut Option<MpvThumbnailRenderer>,
    loaded_url: &mut Option<String>,
    url: &str,
) -> Result<String, String> {
    eprintln!(
        "thumbnail_helper source scheme={} host={} local_torrent={} local_proxy={}",
        url.split_once("://")
            .map(|(scheme, _)| scheme)
            .unwrap_or("unknown"),
        reqwest::Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string()),
        url.contains("/stream/fname"),
        url.contains("/stream-proxy/") || url.contains("/proxy/")
    );
    if renderer.is_none() {
        eprintln!("thumbnail_helper spawn_renderer");
        *renderer = Some(MpvThumbnailRenderer::new()?);
    }
    let renderer = renderer.as_mut().unwrap();
    if loaded_url.as_deref() != Some(url) {
        let load_started = std::time::Instant::now();
        renderer.load_thumbnail(url)?;
        *loaded_url = Some(url.to_string());
        eprintln!(
            "thumbnail_helper load_done elapsed_ms={}",
            load_started.elapsed().as_millis()
        );
    }
    Ok(String::new())
}

fn render_request(
    renderer: &mut Option<MpvThumbnailRenderer>,
    loaded_url: &mut Option<String>,
    url: &str,
    time_pos: f64,
) -> Result<String, String> {
    if !time_pos.is_finite() || time_pos < 0.0 {
        return Err("invalid thumbnail time".to_string());
    }
    if renderer.is_none() {
        *renderer = Some(MpvThumbnailRenderer::new()?);
    }
    let renderer = renderer.as_mut().unwrap();
    if loaded_url.as_deref() != Some(url) {
        renderer.load_thumbnail(url)?;
        *loaded_url = Some(url.to_string());
    }
    let seek_started = std::time::Instant::now();
    renderer.set_paused(false)?;
    renderer.seek_thumbnail_to(time_pos)?;
    renderer.pump_events();
    std::thread::sleep(std::time::Duration::from_millis(50));
    renderer.set_paused(true)?;
    renderer.pump_events();
    std::thread::sleep(std::time::Duration::from_millis(20));
    eprintln!(
        "thumbnail_helper seek_done elapsed_ms={}",
        seek_started.elapsed().as_millis()
    );
    if let Ok((image, has_content)) = encode_frame(renderer) {
        if has_content {
            return Ok(image);
        }
    }
    screenshot_frame(renderer)
}

fn screenshot_frame(renderer: &mut MpvThumbnailRenderer) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "fluxa-seek-thumbnail-{}-{nonce}.jpg",
        std::process::id()
    ));
    let path_string = path.to_string_lossy().into_owned();
    renderer.screenshot_to_file(&path_string)?;
    let mut bytes = Err("mpv did not produce a thumbnail".to_string());
    for _ in 0..3 {
        match std::fs::read(&path) {
            Ok(value) if !value.is_empty() => {
                bytes = Ok(value);
                break;
            }
            _ => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
    }
    let _ = std::fs::remove_file(&path);
    let bytes = bytes?;
    if bytes.is_empty() {
        return Err("mpv returned an empty thumbnail".to_string());
    }
    let mime = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else {
        "image/jpeg"
    };
    Ok(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn encode_frame(renderer: &mut MpvThumbnailRenderer) -> Result<(String, bool), String> {
    use base64::{Engine as _, engine::general_purpose};
    let pixels = renderer.render_thumbnail(320, 180)?;
    let mut min = u8::MAX;
    let mut max = u8::MIN;
    let mut non_black = 0usize;
    for pixel in pixels.chunks_exact(4) {
        let luminance =
            ((u32::from(pixel[0]) * 299 + u32::from(pixel[1]) * 587 + u32::from(pixel[2]) * 114)
                / 1000) as u8;
        min = min.min(luminance);
        max = max.max(luminance);
        if luminance > 8 {
            non_black += 1;
        }
    }
    let has_content = max.saturating_sub(min) > 10 && non_black > pixels.len() / 32;
    let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(320, 180, pixels)
        .ok_or_else(|| "frame buffer mismatch".to_string())?;
    let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
    let mut jpeg = Vec::new();
    rgb.write_to(
        &mut std::io::Cursor::new(&mut jpeg),
        image::ImageFormat::Jpeg,
    )
    .map_err(|error| error.to_string())?;
    Ok((
        format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(jpeg)
        ),
        has_content,
    ))
}
