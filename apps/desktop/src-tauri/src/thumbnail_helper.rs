use crate::mpv_render::MpvThumbnailRenderer;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Deserialize, Serialize)]
struct ThumbnailRequest {
    url: String,
    time_pos: f64,
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
            .stderr(Stdio::null())
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
        let request = serde_json::to_string(&ThumbnailRequest {
            url: url.to_string(),
            time_pos,
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
        if response.ok {
            response
                .image
                .ok_or_else(|| "thumbnail helper returned no image".to_string())
        } else {
            Err(response.error.unwrap_or_else(|| "thumbnail failed".to_string()))
        }
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
                Ok(request) => match render_request(
                    &mut renderer,
                    &mut loaded_url,
                    &request.url,
                    request.time_pos,
                ) {
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
        for _ in 0..50 {
            if renderer.query_property("duration").is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    renderer.seek_to(time_pos)?;
    for _ in 0..300 {
        if renderer.query_property("seeking").as_deref() != Some("yes") {
            return encode_frame(renderer);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Err("thumbnail not ready".to_string())
}

fn encode_frame(renderer: &mut MpvThumbnailRenderer) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};
    let pixels = renderer.render_thumbnail(320, 180)?;
    let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(320, 180, pixels)
        .ok_or_else(|| "frame buffer mismatch".to_string())?;
    let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
    let mut jpeg = Vec::new();
    rgb.write_to(
        &mut std::io::Cursor::new(&mut jpeg),
        image::ImageFormat::Jpeg,
    )
    .map_err(|error| error.to_string())?;
    Ok(format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(jpeg)
    ))
}
