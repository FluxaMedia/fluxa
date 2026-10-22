use super::watch_together::ActionSink;
use super::*;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom, Write};
use std::sync::Mutex;
use tokio::task::AbortHandle;

const TICK: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(super) struct Downloads {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    sink: Option<ActionSink>,
    tasks: HashMap<String, AbortHandle>,
}

impl Downloads {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn emit(&self, action: Value) {
        let sink = self.lock().sink.clone();
        if let Some(sink) = sink {
            sink(action);
        }
    }
}

fn now_ms() -> u64 {
    u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or_default()
}

fn extension(url: &str, stream: &Value) -> String {
    let named = stream
        .pointer("/behaviorHints/filename")
        .and_then(Value::as_str);
    let path = url.split(['?', '#']).next().unwrap_or(url);
    named
        .into_iter()
        .chain(path.rsplit('/').next())
        .filter_map(|name| name.rsplit_once('.').map(|(_, ext)| ext))
        .find(|ext| (2..=4).contains(&ext.len()) && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("mkv")
        .to_ascii_lowercase()
}

impl EffectExecutor {
    pub fn set_download_sink(&self, sink: ActionSink) {
        self.downloads.lock().sink = Some(sink);
    }

    pub fn torrent_http_url(&self, stream: &Value, title: &str) -> Result<String, String> {
        let link = stream
            .get("infoHash")
            .and_then(Value::as_str)
            .filter(|hash| !hash.is_empty())
            .map(|hash| match stream.get("fileIdx").and_then(Value::as_i64) {
                Some(index) => format!("stremio://torrent/{hash}/{index}"),
                None => format!("stremio://torrent/{hash}"),
            })
            .ok_or_else(|| "stream has no torrent source".to_owned())?;
        let started = self.start_torrent_stream(&json!({
            "url": link,
            "stream": stream,
            "title": title,
            "fileIdx": stream.get("fileIdx"),
            "preferredFilename": stream.pointer("/behaviorHints/filename"),
            "sources": stream.get("sources"),
        }))?;
        started
            .get("url")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| "torrent stream did not return a url".to_owned())
    }

    fn downloads_dir(&self) -> std::path::PathBuf {
        self.storage.dir().join("downloads")
    }

    fn download_dir(&self, payload: &Value) -> std::path::PathBuf {
        payload
            .get("dir")
            .and_then(Value::as_str)
            .filter(|dir| !dir.is_empty())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| self.downloads_dir())
    }

    pub(super) fn read_download_storage(&self, payload: &Value) -> Result<Value, String> {
        let default = self.downloads_dir();
        let current = self.download_dir(payload);
        let _ = std::fs::create_dir_all(&current);
        let mut volumes = vec![volume(&default, "default")];
        for root in mounted_roots() {
            let root = root.join("Fluxa");
            if !volumes.iter().any(|known| known["path"] == root.to_string_lossy().as_ref()) {
                let label = root
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_owned();
                volumes.push(volume(&root, &label));
            }
        }
        let label = if current == default { "default" } else { "custom" };
        Ok(json!({"location": volume(&current, label), "volumes": volumes}))
    }

    pub(super) fn start_offline_download(&self, payload: &Value) -> Result<Value, String> {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| "download payload is missing id".to_owned())?
            .to_owned();
        let stream = payload.get("stream").cloned().unwrap_or(Value::Null);
        let source = payload
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .ok_or_else(|| "download payload is missing a url".to_owned())?;
        let url = if source.starts_with("http://") || source.starts_with("https://") {
            source.to_owned()
        } else {
            let name = payload
                .pointer("/meta/name")
                .and_then(Value::as_str)
                .unwrap_or("Fluxa");
            self.torrent_http_url(&stream, name)?
        };
        let dir = self.download_dir(payload);
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let stem = sanitize_key(&id);
        let part = dir.join(format!("{stem}.part"));
        let target = dir.join(format!("{stem}.{}", extension(&url, &stream)));

        let downloads = self.downloads.clone();
        let task_id = id.clone();
        let task = super::runtime().spawn(async move {
            let outcome = download(&downloads, &task_id, &url, &stream, &part, &target).await;
            downloads.lock().tasks.remove(&task_id);
            match outcome {
                Ok((path, total)) => downloads.emit(json!({
                    "type": "offlineDownloadCompleted",
                    "id": task_id,
                    "path": path,
                    "bytesTotal": total,
                    "nowMs": now_ms(),
                })),
                Err(error) => downloads.emit(json!({
                    "type": "offlineDownloadFailed",
                    "id": task_id,
                    "error": error,
                })),
            }
        });
        if let Some(previous) = self
            .downloads
            .lock()
            .tasks
            .insert(id.clone(), task.abort_handle())
        {
            previous.abort();
        }
        Ok(json!({"id": id}))
    }

    pub(super) fn control_offline_download(&self, payload: &Value) -> Result<Value, String> {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "download control is missing id".to_owned())?;
        if let Some(task) = self.downloads.lock().tasks.remove(id) {
            task.abort();
        }
        if payload.get("command").and_then(Value::as_str) == Some("remove") {
            let stem = sanitize_key(id);
            if let Ok(entries) = std::fs::read_dir(self.download_dir(payload)) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.file_stem().and_then(|name| name.to_str()) == Some(stem.as_str()) {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
        Ok(json!({"id": id}))
    }
}

async fn download(
    downloads: &Downloads,
    id: &str,
    url: &str,
    stream: &Value,
    part: &std::path::Path,
    target: &std::path::Path,
) -> Result<(String, Option<u64>), String> {
    let mut have = std::fs::metadata(part).map(|meta| meta.len()).unwrap_or(0);
    let mut request = Client::builder()
        .bundled_roots()
        .connect_timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?
        .get(url);
    if let Some(headers) = stream
        .pointer("/behaviorHints/proxyHeaders/request")
        .and_then(Value::as_object)
    {
        for (name, value) in headers {
            if let Some(value) = value.as_str() {
                request = request.header(name.as_str(), value);
            }
        }
    }
    if have > 0 {
        request = request.header("Range", format!("bytes={have}-"));
    }
    let mut response = request.send().await.map_err(|error| error.to_string())?;
    if response.status().as_u16() == 416 && have > 0 {
        std::fs::rename(part, target).map_err(|error| error.to_string())?;
        return Ok((target.to_string_lossy().into_owned(), Some(have)));
    }
    if !response.status().is_success() {
        return Err(format!("http_{}", response.status().as_u16()));
    }
    let resumed = response.status().as_u16() == 206;
    if !resumed {
        have = 0;
    }
    let total = response
        .content_length()
        .map(|length| length + have)
        .filter(|total| *total > 0);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(!resumed)
        .open(part)
        .map_err(|error| error.to_string())?;
    if resumed {
        file.seek(SeekFrom::Start(have))
            .map_err(|error| error.to_string())?;
    }

    if let Some(total) = total
        && let Some(free) = part.parent().and_then(free_bytes)
        && free < total.saturating_sub(have)
    {
        return Err("no_space".to_owned());
    }

    let mut done = have;
    let mut last = std::time::Instant::now() - TICK;
    loop {
        match response.chunk().await.map_err(|error| error.to_string())? {
            Some(chunk) => {
                if done == 0 && is_playlist(&chunk) {
                    drop(file);
                    let _ = std::fs::remove_file(part);
                    return hls(downloads, id, url, stream, &target.with_extension("mkv"))
                        .await;
                }
                file.write_all(&chunk).map_err(write_error)?;
                done += chunk.len() as u64;
            }
            None => break,
        }
        if last.elapsed() >= TICK {
            last = std::time::Instant::now();
            downloads.emit(json!({
                "type": "offlineDownloadProgressed",
                "id": id,
                "bytesDone": done,
                "bytesTotal": total,
                "nowMs": now_ms(),
            }));
        }
    }
    file.flush().map_err(|error| error.to_string())?;
    drop(file);
    if let Some(total) = total
        && done < total
    {
        return Err("connection_closed".to_owned());
    }
    std::fs::rename(part, target).map_err(|error| error.to_string())?;
    Ok((target.to_string_lossy().into_owned(), Some(done)))
}

struct CancelOnDrop(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

async fn hls(
    downloads: &Downloads,
    id: &str,
    url: &str,
    stream: &Value,
    target: &std::path::Path,
) -> Result<(String, Option<u64>), String> {
    let headers: Vec<(String, String)> = stream
        .pointer("/behaviorHints/proxyHeaders/request")
        .and_then(Value::as_object)
        .map(|headers| {
            headers
                .iter()
                .filter_map(|(name, value)| Some((name.clone(), value.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    let cancel = CancelOnDrop(Default::default());
    let flag = cancel.0.clone();
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let (url, output) = (url.to_owned(), target.to_path_buf());
    let worker = tokio::task::spawn_blocking(move || {
        let mut last = std::time::Instant::now() - TICK;
        fluxa_mpv::remux(&url, &headers, &output, &flag, |progress| {
            if last.elapsed() >= TICK {
                last = std::time::Instant::now();
                let _ = sender.send(progress);
            }
        })
    });
    while let Some(progress) = receiver.recv().await {
        let total = progress
            .duration
            .filter(|duration| progress.seconds > 1.0 && *duration >= progress.seconds)
            .map(|duration| (progress.bytes as f64 * duration / progress.seconds) as u64)
            .filter(|total| *total >= progress.bytes);
        downloads.emit(json!({
            "type": "offlineDownloadProgressed",
            "id": id,
            "bytesDone": progress.bytes,
            "bytesTotal": total,
            "nowMs": now_ms(),
        }));
    }
    let outcome = worker.await.map_err(|error| error.to_string())?;
    if let Err(error) = outcome {
        let _ = std::fs::remove_file(target);
        return Err(if error.contains("libmpv") {
            "playlist_stream".to_owned()
        } else {
            "remux_failed".to_owned()
        });
    }
    let done = std::fs::metadata(target).map(|meta| meta.len()).unwrap_or(0);
    Ok((target.to_string_lossy().into_owned(), Some(done)))
}

fn is_playlist(chunk: &[u8]) -> bool {
    chunk.trim_ascii_start().starts_with(b"#EXTM3U")
}

fn write_error(error: std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::StorageFull {
        "no_space".to_owned()
    } else {
        error.to_string()
    }
}

fn volume(path: &std::path::Path, label: &str) -> Value {
    let (free, total) = path
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .and_then(space)
        .unzip();
    json!({
        "path": path.to_string_lossy(),
        "label": label,
        "freeBytes": free.unwrap_or_default(),
        "totalBytes": total.unwrap_or_default(),
    })
}

fn free_bytes(path: &std::path::Path) -> Option<u64> {
    space(path).map(|(free, _)| free)
}

#[cfg(unix)]
fn space(path: &std::path::Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::zeroed();
    if unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }
    let stat = unsafe { stat.assume_init() };
    let block = stat.f_frsize as u64;
    Some((stat.f_bavail as u64 * block, stat.f_blocks as u64 * block))
}

#[cfg(windows)]
fn space(path: &std::path::Path) -> Option<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            dir: *const u16,
            available: *mut u64,
            total: *mut u64,
            free: *mut u64,
        ) -> i32;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let (mut available, mut total, mut free) = (0u64, 0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut free) };
    (ok != 0).then_some((available, total))
}

#[cfg(not(any(unix, windows)))]
fn space(_path: &std::path::Path) -> Option<(u64, u64)> {
    None
}

#[cfg(target_os = "linux")]
fn mounted_roots() -> Vec<std::path::PathBuf> {
    let mounts = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    mounts
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(|path| path.replace("\\040", " "))
        .filter(|path| {
            ["/media/", "/mnt/", "/run/media/"]
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .map(std::path::PathBuf::from)
        .collect()
}

#[cfg(target_os = "macos")]
fn mounted_roots() -> Vec<std::path::PathBuf> {
    std::fs::read_dir("/Volumes")
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default()
}

#[cfg(windows)]
fn mounted_roots() -> Vec<std::path::PathBuf> {
    ('A'..='Z')
        .map(|letter| std::path::PathBuf::from(format!("{letter}:\\")))
        .filter(|root| root.exists())
        .collect()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn mounted_roots() -> Vec<std::path::PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hls_playlists_are_recognised_by_their_first_bytes() {
        assert!(is_playlist(b"\xef\xbb\xbf#EXTM3U\n".get(3..).unwrap()));
        assert!(is_playlist(b"  \n#EXTM3U"));
        assert!(!is_playlist(&[0x1a, 0x45, 0xdf, 0xa3]));
    }

    #[test]
    fn hls_playlist_is_remuxed_into_one_mkv() {
        let dir = std::env::current_dir().unwrap().join("hls-test-output");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let made = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-f", "lavfi", "-i", "testsrc=d=3:s=160x90:r=10"])
            .args(["-f", "lavfi", "-i", "sine=d=3", "-c:v", "mpeg2video", "-c:a", "mp2"])
            .args(["-f", "hls", "-hls_time", "1", "-hls_playlist_type", "vod"])
            .arg(dir.join("index.m3u8"))
            .status();
        let target = dir.join("out.mkv");
        let outcome = made.is_ok_and(|status| status.success()).then(|| {
            crate::executor::runtime().block_on(hls(
                &Downloads::default(),
                "id",
                dir.join("index.m3u8").to_str().unwrap(),
                &Value::Null,
                &target,
            ))
        });
        let size = std::fs::metadata(&target).map(|meta| meta.len()).unwrap_or(0);
        let _ = std::fs::remove_dir_all(&dir);
        match outcome {
            Some(Ok((path, total))) => {
                assert!(path.ends_with("out.mkv"));
                assert!(size > 0 && total == Some(size));
            }
            Some(Err(error)) => assert_eq!(error, "playlist_stream"),
            None => {}
        }
    }

    #[test]
    fn missing_folders_report_the_space_of_their_nearest_parent() {
        let missing = std::env::current_dir().unwrap().join("not-created-yet/Fluxa");
        let info = volume(&missing, "custom");
        assert!(info["totalBytes"].as_u64().unwrap() > 0);
        assert!(info["freeBytes"].as_u64().unwrap() <= info["totalBytes"].as_u64().unwrap());
    }
}
