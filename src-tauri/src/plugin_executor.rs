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
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => return failed_response(error.to_string()),
    };
    runtime.block_on(async move {
        let method = match reqwest::Method::from_bytes(request.method.as_bytes()) {
            Ok(method) => method,
            Err(error) => return failed_response(error.to_string()),
        };
        let mut url = match reqwest::Url::parse(&request.url) {
            Ok(url) => url,
            Err(error) => return failed_response(error.to_string()),
        };
        let mut redirects_left = request.follow_redirects.then_some(10).unwrap_or(0);
        loop {
            let client = match crate::net_guard::vetted_client_without_redirects(
                url.as_str(),
                Duration::from_secs(FETCH_TIMEOUT_SECS),
            )
            .await
            {
                Ok(client) => client,
                Err(error) => return failed_response(error),
            };
            let mut outgoing = client.request(method.clone(), url.clone());
            for (name, value) in &request.headers {
                let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) else {
                    continue;
                };
                outgoing = outgoing.header(name, value);
            }
            if let Some(body) = &request.body {
                outgoing = outgoing.body(body.clone());
            }
            let response = match outgoing.send().await {
                Ok(response) => response,
                Err(error) => return failed_response(error.to_string()),
            };
            if response.status().is_redirection() && redirects_left > 0 {
                let Some(location) = response.headers().get(reqwest::header::LOCATION) else {
                    return failed_response("redirect without location".to_string());
                };
                let location = match location.to_str() {
                    Ok(location) => location,
                    Err(_) => return failed_response("redirect has invalid location".to_string()),
                };
                url = match url.join(location) {
                    Ok(url) => url,
                    Err(error) => return failed_response(error.to_string()),
                };
                redirects_left -= 1;
                continue;
            }
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.to_string(), value.to_string()))
                })
                .collect();
            let body = response.text().await.unwrap_or_default();
            return PluginHttpResponse {
                status,
                headers,
                body,
                ok: (200..300).contains(&status),
                error: None,
            };
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
