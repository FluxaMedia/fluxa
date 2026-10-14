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

    fn downloads_dir(&self) -> std::path::PathBuf {
        self.storage.dir().join("downloads")
    }

    pub(super) fn start_offline_download(&self, payload: &Value) -> Result<Value, String> {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| "download payload is missing id".to_owned())?
            .to_owned();
        let url = payload
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| url.starts_with("http://") || url.starts_with("https://"))
            .ok_or_else(|| "download payload is missing an http url".to_owned())?
            .to_owned();
        let stream = payload.get("stream").cloned().unwrap_or(Value::Null);
        let dir = self.downloads_dir();
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
            if let Ok(entries) = std::fs::read_dir(self.downloads_dir()) {
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

    let mut done = have;
    let mut last = std::time::Instant::now() - TICK;
    loop {
        match response.chunk().await.map_err(|error| error.to_string())? {
            Some(chunk) => {
                file.write_all(&chunk).map_err(|error| error.to_string())?;
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
