use super::*;

impl EffectExecutor {
    #[cfg(target_arch = "wasm32")]
    pub fn warm_torrent_engine(&self) {}

    #[cfg(not(target_arch = "wasm32"))]
    pub fn warm_torrent_engine(&self) {
        std::thread::spawn(|| match ensure_torrent_server() {
            Ok(_) => crate::log!("[fluxa-native] torrent engine warmed up"),
            Err(error) => crate::log!("[fluxa-native] torrent engine warm-up failed: {error}"),
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_torrent_status(
        &self,
        _link: String,
        _file_id: Option<usize>,
    ) -> std::sync::mpsc::Receiver<Value> {
        std::sync::mpsc::channel().1
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_torrent_status(
        &self,
        link: String,
        file_id: Option<usize>,
    ) -> std::sync::mpsc::Receiver<Value> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let Ok(server) = ensure_torrent_server() else {
                return;
            };
            let Some(base_url) = server.get("url").and_then(Value::as_str) else {
                return;
            };
            let url = format!("{}/torrents", base_url.trim_end_matches('/'));
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let client = Client::new();
            loop {
                let response: Result<Value, String> = runtime
                    .block_on(async {
                        client
                            .post(&url)
                            .json(&json!({ "action": "get", "link": link, "file_id": file_id }))
                            .timeout(Duration::from_secs(3))
                            .send()
                            .await?
                            .error_for_status()?
                            .json::<Value>()
                            .await
                    })
                    .map_err(|error: reqwest::Error| error.to_string());
                let status = response
                    .unwrap_or_else(|error| json!({"phase": "status_unavailable", "error": error}));
                if sender.send(status).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(700));
            }
        });
        receiver
    }

    pub(super) fn start_torrent_stream(&self, payload: &Value) -> Result<Value, String> {
        let stream = payload
            .get("stream")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "torrent stream payload is missing stream".to_owned())?;
        let link = payload
            .get("url")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "torrent stream payload is missing url".to_owned())?;
        let title = payload
            .get("title")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Fluxa stream");
        let server = ensure_torrent_server()?;
        let base_url = server
            .get("url")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "torrent server did not return url".to_owned())?;
        let runtime_request = json!({
            "link": link,
            "title": title,
            "requestedFileIdx": payload.get("fileIdx").cloned().unwrap_or(Value::Null),
            "preferredFilename": payload.get("preferredFilename").cloned().unwrap_or(Value::Null),
            "sources": payload.get("sources").cloned().unwrap_or_else(|| json!([])),
            "fileStats": [],
            "rejectedIndex": null,
            "baseUrl": base_url,
            "play": true,
            "stat": false,
            "durationMs": null,
        });
        let runtime = core_value("torrentRuntimeInfo", runtime_request)
            .ok_or_else(|| "torrent runtime info could not be resolved".to_owned())?;
        let stream_url = runtime
            .get("streamUrl")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "torrent runtime did not return streamUrl".to_owned())?;
        let normalized_link = runtime
            .get("normalizedLink")
            .and_then(Value::as_str)
            .unwrap_or(link)
            .to_owned();
        let file_id = runtime.get("selectedFileIdx").and_then(Value::as_i64);
        crate::log!(
            "[fluxa-native] torrent stream ready base={} url={}",
            base_url,
            stream_url
        );
        start_torrent_add(base_url.to_owned(), normalized_link, file_id);
        Ok(json!({
            "url": stream_url,
            "baseUrl": base_url,
            "generation": server.get("generation").cloned().unwrap_or(Value::Null),
            "stream": stream,
        }))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) static TORRENT_SERVER: OnceLock<std::sync::Mutex<Option<Value>>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
pub(super) fn ensure_torrent_server() -> Result<Value, String> {
    Err("torrent streaming is not available on this platform".to_owned())
}

#[cfg(target_arch = "wasm32")]
pub(super) fn start_torrent_add(_base_url: String, _link: String, _file_id: Option<i64>) {}

pub(super) static TORRENT_CACHE_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn ensure_torrent_server() -> Result<Value, String> {
    let slot = TORRENT_SERVER.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = slot
        .lock()
        .map_err(|_| "torrent server state is poisoned".to_owned())?;
    if let Some(server) = guard.as_ref() {
        return Ok(server.clone());
    }
    let cache_dir = TORRENT_CACHE_DIR
        .get()
        .cloned()
        .ok_or_else(|| "torrent cache directory is not configured".to_owned())?;
    let server_raw =
        fluxa_streaming_engine::start_torrent_server(&cache_dir.to_string_lossy(), 0, "")
            .ok_or_else(|| "failed to start torrent server".to_owned())?;
    let server: Value = serde_json::from_str(&server_raw)
        .map_err(|error| format!("invalid torrent server response: {error}"))?;
    *guard = Some(server.clone());
    Ok(server)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn start_torrent_add(base_url: String, link: String, file_id: Option<i64>) {
    std::thread::spawn(move || {
        let info_hash = link
            .split("xt=urn:btih:")
            .nth(1)
            .map(|tail| {
                tail.chars()
                    .take_while(char::is_ascii_alphanumeric)
                    .collect::<String>()
            })
            .unwrap_or_default();
        crate::log!(
            "[fluxa-native] torrent metadata add started hash={info_hash} file={file_id:?}"
        );
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        let result: Result<Value, String> = runtime.block_on(async {
            let response = Client::new()
                .post(format!("{}/torrents", base_url.trim_end_matches('/')))
                .json(&json!({ "action": "add", "link": link, "file_id": file_id }))
                .timeout(Duration::from_secs(95))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            let status = response.status();
            let body = response.text().await.map_err(|error| error.to_string())?;
            if !status.is_success() {
                return Err(format!("HTTP {status}: {body}"));
            }
            serde_json::from_str(&body).map_err(|error| error.to_string())
        });
        match result {
            Ok(status) => crate::log!(
                "[fluxa-native] torrent metadata ready hash={} peers={}",
                status.get("hash").and_then(Value::as_str).unwrap_or(""),
                status
                    .get("active_peers")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            ),
            Err(error) => crate::log!("[fluxa-native] torrent add failed: {error}"),
        }
    });
}
