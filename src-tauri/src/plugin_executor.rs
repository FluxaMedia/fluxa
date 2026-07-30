use fluxa_core::plugin_runtime::{PluginHttpClient, PluginHttpRequest, PluginHttpResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

const FETCH_TIMEOUT_SECS: u64 = 15;

pub fn execute_scraper(
    code: String,
    scraper_id: String,
    scraper_settings_json: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    fluxa_core::plugin_runtime::execute_scraper(
        Arc::new(DesktopPluginHttpClient),
        code,
        scraper_id,
        scraper_settings_json,
        tmdb_id,
        media_type,
        season,
        episode,
    )
}

struct DesktopPluginHttpClient;

impl PluginHttpClient for DesktopPluginHttpClient {
    fn fetch(&self, request: PluginHttpRequest) -> PluginHttpResponse {
        std::thread::spawn(move || fetch(request))
            .join()
            .unwrap_or_else(|_| failed_response("plugin request worker panicked".to_string()))
    }
}

fn fetch(request: PluginHttpRequest) -> PluginHttpResponse {
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => return failed_response(error.to_string()),
    };
    runtime.block_on(async move {
        let client = match crate::net_guard::vetted_client(&request.url, Duration::from_secs(FETCH_TIMEOUT_SECS)).await {
            Ok(client) => client,
            Err(error) => return failed_response(error),
        };
        let method = match reqwest::Method::from_bytes(request.method.as_bytes()) {
            Ok(method) => method,
            Err(error) => return failed_response(error.to_string()),
        };
        let mut outgoing = client.request(method, &request.url);
        for (name, value) in request.headers {
            let (Ok(name), Ok(value)) = (
                reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                reqwest::header::HeaderValue::from_str(&value),
            ) else {
                continue;
            };
            outgoing = outgoing.header(name, value);
        }
        if let Some(body) = request.body {
            outgoing = outgoing.body(body);
        }
        match outgoing.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let headers = response.headers().iter().filter_map(|(name, value)| {
                    value.to_str().ok().map(|value| (name.to_string(), value.to_string()))
                }).collect();
                let body = response.text().await.unwrap_or_default();
                PluginHttpResponse {
                    status,
                    headers,
                    body,
                    ok: (200..300).contains(&status),
                    error: None,
                }
            }
            Err(error) => failed_response(error.to_string()),
        }
    })
}

fn failed_response(error: String) -> PluginHttpResponse {
    PluginHttpResponse {
        status: 0,
        headers: HashMap::new(),
        body: String::new(),
        ok: false,
        error: Some(error),
    }
}
