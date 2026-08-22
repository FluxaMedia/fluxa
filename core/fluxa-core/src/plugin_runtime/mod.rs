mod crypto_bridge;
mod dom_bridge;
mod host_functions;
mod scraper_exec;
mod settings_layout;
mod web_compat;

use scraper_exec::run;
use serde::{Deserialize, Serialize};
use settings_layout::run_settings_layout;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};
use url::{Host, Url};

pub(super) const PLUGIN_TIMEOUT_SECS: u64 = 60;
pub(super) const PLUGIN_MEMORY_LIMIT: usize = 256 * 1024 * 1024;
pub const PLUGIN_MAX_REQUEST_BODY_BYTES: usize = 1_048_576;
pub const PLUGIN_MAX_RESPONSE_BODY_BYTES: usize = 8 * 1_048_576;

pub(super) fn plugin_memory_limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("FLUXA_PLUGIN_MEMORY_LIMIT_MB")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .map(|mb| mb.clamp(32, 512) * 1024 * 1024)
            .unwrap_or(PLUGIN_MEMORY_LIMIT)
    })
}

fn plugin_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}

struct ScraperJob {
    client: Arc<dyn PluginHttpClient>,
    code: String,
    repository_url: String,
    scraper_id: String,
    scraper_settings_json: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
    result: Sender<Result<String, String>>,
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
}

struct ScraperWorker {
    jobs: tokio::sync::mpsc::Sender<ScraperJob>,
}

fn plugin_worker_count() -> usize {
    let default = if cfg!(target_os = "android") {
        1
    } else {
        std::thread::available_parallelism()
            .map(|cores| cores.get().clamp(1, 4))
            .unwrap_or(2)
    };
    std::env::var("FLUXA_PLUGIN_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (1..=8).contains(value))
        .unwrap_or(default)
}

fn scraper_workers() -> &'static [ScraperWorker] {
    static WORKERS: OnceLock<Vec<ScraperWorker>> = OnceLock::new();
    WORKERS.get_or_init(|| {
        (0..plugin_worker_count())
            .filter_map(|index| {
                let (jobs, mut receiver) = tokio::sync::mpsc::channel::<ScraperJob>(4);
                let name = format!("fluxa-plugin-worker-{index}");
                std::thread::Builder::new()
                    .name(name)
                    .spawn(move || {
                        let runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .ok()?;
                        let local = tokio::task::LocalSet::new();
                        local.block_on(&runtime, async move {
                            while let Some(job) = receiver.recv().await {
                                if job.cancelled.load(Ordering::Acquire)
                                    || Instant::now() >= job.deadline
                                {
                                    let _ = job.result.send(Err("plugin job expired".to_string()));
                                    continue;
                                }
                                let remaining =
                                    job.deadline.saturating_duration_since(Instant::now());
                                let cancelled = job.cancelled.clone();
                                let result = tokio::select! {
                                    result = tokio::time::timeout(
                                        remaining,
                                        run(
                                            job.client,
                                            job.code,
                                            format!("{}\u{1f}{}", job.repository_url, job.scraper_id),
                                            job.scraper_id,
                                            job.scraper_settings_json,
                                            job.tmdb_id,
                                            job.media_type,
                                            job.season,
                                            job.episode,
                                        ),
                                    ) => result.unwrap_or_else(|_| Err("plugin timed out".to_string())),
                                    _ = async {
                                        while !cancelled.load(Ordering::Acquire) {
                                            tokio::time::sleep(Duration::from_millis(50)).await;
                                        }
                                    } => Err("plugin job cancelled".to_string()),
                                };
                                let _ = job.result.send(result);
                            }
                        });
                        Some(())
                    })
                    .ok()?;
                Some(ScraperWorker { jobs })
            })
            .collect()
    })
}

pub fn plugin_http_request_error(request: &PluginHttpRequest) -> Option<&'static str> {
    if !matches!(
        request.method.to_ascii_uppercase().as_str(),
        "GET" | "HEAD" | "POST"
    ) {
        return Some("plugin request method is not allowed");
    }
    let parsed = match Url::parse(&request.url) {
        Ok(parsed) => parsed,
        Err(_) => return Some("invalid plugin URL"),
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return Some("only http and https plugin URLs are allowed");
    }
    let Some(host) = parsed.host() else {
        return Some("plugin URL is missing a host");
    };
    if !is_public_plugin_host(host) {
        return Some("private or loopback plugin URL is not allowed");
    }
    if let Some(body) = &request.body
        && body.len() > PLUGIN_MAX_REQUEST_BODY_BYTES
    {
        return Some("plugin request body exceeds limit");
    }
    // Nuvio-compatible scrapers commonly need Cookie, Authorization and X-* headers.
    // Keep SSRF protection at the URL/DNS layer and only reject transport headers
    // that plugins must not be allowed to forge.
    if request.headers.keys().any(|key| {
        matches!(
            key.trim().to_ascii_lowercase().as_str(),
            "host"
                | "content-length"
                | "connection"
                | "transfer-encoding"
                | "upgrade"
                | "proxy-authorization"
                | "proxy-authenticate"
                | "te"
                | "trailer"
        )
    }) {
        return Some("plugin request contains a protected transport header");
    }
    None
}

fn is_public_plugin_host(host: Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => {
            !ip.is_private() && !ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified()
        }
        Host::Ipv6(ip) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
        }
        Host::Domain(domain) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            domain != "localhost"
                && !domain.ends_with(".localhost")
                && domain
                    .parse::<std::net::Ipv4Addr>()
                    .map(|ip| is_public_plugin_host(Host::Ipv4(ip)))
                    .unwrap_or(true)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PluginHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub follow_redirects: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PluginHttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub ok: bool,
    pub error: Option<String>,
}

#[cfg_attr(feature = "uniffi-bindings", uniffi::export(callback_interface))]
pub trait PluginHttpClient: Send + Sync {
    /// Before every connection, resolve the requested host and reject every
    /// loopback, private, link-local, or unspecified A/AAAA result. Apply the
    /// same check to every redirect target to prevent DNS rebinding and redirect
    /// based SSRF; this core-side policy can only inspect the original URL.
    fn fetch(&self, request: PluginHttpRequest) -> PluginHttpResponse;
}

#[expect(
    clippy::too_many_arguments,
    reason = "public scraper bridge mirrors the established FFI wire contract"
)]
pub fn execute_scraper(
    client: Arc<dyn PluginHttpClient>,
    code: String,
    repository_url: String,
    scraper_id: String,
    scraper_settings_json: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    static NEXT_WORKER: AtomicUsize = AtomicUsize::new(0);
    let workers = scraper_workers();
    if workers.is_empty() {
        return Err("plugin worker pool unavailable".to_string());
    }
    let worker = workers
        .get(NEXT_WORKER.fetch_add(1, Ordering::Relaxed) % workers.len())
        .ok_or_else(|| "plugin worker pool unavailable".to_string())?;
    let (result, receiver) = mpsc::channel();
    let cancelled = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_secs(PLUGIN_TIMEOUT_SECS + 5);
    let job = ScraperJob {
        client,
        code,
        repository_url,
        scraper_id,
        scraper_settings_json,
        tmdb_id,
        media_type,
        season,
        episode,
        result,
        deadline,
        cancelled: cancelled.clone(),
    };
    let mut job = Some(job);
    loop {
        match worker.jobs.try_reserve() {
            Ok(permit) => {
                permit.send(job.take().expect("plugin job is present"));
                break;
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(())) => {
                if Instant::now() >= deadline {
                    cancelled.store(true, Ordering::Release);
                    return Err("plugin worker queue wait timed out".to_string());
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(())) => {
                return Err("plugin worker pool closed".to_string());
            }
        }
    }
    match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            cancelled.store(true, Ordering::Release);
            Err("plugin worker result timed out".to_string())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err("plugin worker pool closed".to_string()),
    }
}

pub fn get_settings_layout(code: String, scraper_id: String) -> String {
    let Ok(rt) = plugin_runtime() else {
        return "[]".to_string();
    };
    let local = tokio::task::LocalSet::new();
    local.block_on(rt, async move {
        tokio::time::timeout(
            Duration::from_secs(PLUGIN_TIMEOUT_SECS),
            run_settings_layout(code, scraper_id),
        )
        .await
        .unwrap_or_else(|_| "[]".to_string())
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    struct MockHttpClient;

    impl PluginHttpClient for MockHttpClient {
        fn fetch(&self, _request: PluginHttpRequest) -> PluginHttpResponse {
            PluginHttpResponse {
                status: 0,
                headers: HashMap::new(),
                body: String::new(),
                ok: false,
                error: Some("network disabled in unit tests".to_string()),
            }
        }
    }

    fn mock_client() -> Arc<dyn PluginHttpClient> {
        Arc::new(MockHttpClient)
    }

    #[test]
    fn plugin_request_policy_rejects_private_hosts_and_credentials() {
        let mut request = PluginHttpRequest {
            url: "http://127.0.0.1/admin".to_string(),
            ..PluginHttpRequest::default()
        };
        assert!(plugin_http_request_error(&request).is_some());
        request.url = "https://example.com/".to_string();
        request
            .headers
            .insert("Authorization".to_string(), "secret".to_string());
        assert!(plugin_http_request_error(&request).is_some());
    }

    #[test]
    fn plugin_request_policy_rejects_bracketed_private_ipv6() {
        for url in [
            "http://[::1]/",
            "http://[fc00::1]/",
            "http://[fe80::1]/",
            "http://localhost./",
            "http://127.0.0.1./",
        ] {
            let request = PluginHttpRequest {
                url: url.to_string(),
                ..PluginHttpRequest::default()
            };
            assert!(plugin_http_request_error(&request).is_some(), "{url}");
        }
    }

    fn run_scraper(code: &str, tmdb_id: &str, media_type: &str) -> String {
        execute_scraper(
            mock_client(),
            code.to_string(),
            "test-repository".to_string(),
            "test-scraper".to_string(),
            "{}".to_string(),
            tmdb_id.to_string(),
            media_type.to_string(),
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn scraper_can_use_cheerio_and_module_exports_without_network() {
        let code = r#"
            module.exports.getStreams = async function(tmdbId, mediaType) {
                var $ = cheerio.load('<div class="row" data-q="1080p">Alpha</div>');
                var el = $('.row');
                return [{ title: el.attr('data-q'), url: 'https://example.com/' + tmdbId + '-' + mediaType }];
            };
        "#;

        let result = run_scraper(code, "123", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["title"], "1080p");
        assert_eq!(parsed[0]["url"], "https://example.com/123-movie");
    }

    #[test]
    fn scraper_that_throws_returns_empty_array_instead_of_erroring() {
        let code = r#"
            module.exports.getStreams = async function() {
                throw new Error("boom");
            };
        "#;

        let result = run_scraper(code, "1", "movie");
        assert_eq!(result, "[]");
    }

    #[test]
    fn fetch_bridge_surfaces_host_rejection_as_not_ok() {
        let code = r#"
            module.exports.getStreams = async function() {
                var res = await fetch('https://example.com/secret');
                return [{ title: 'x', url: 'https://example.com/blocked', ok: res.ok }];
            };
        "#;

        let result = run_scraper(code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["ok"], false);
    }

    #[test]
    fn cryptojs_sha256_matches_known_vector() {
        let code = r#"
            module.exports.getStreams = async function() {
                var hash = CryptoJS.SHA256('abc').toString(CryptoJS.enc.Hex);
                return [{ title: hash, url: 'https://example.com/x' }];
            };
        "#;
        let result = run_scraper(code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed[0]["title"],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn cryptojs_hmac_sha256_matches_known_vector() {
        let code = r#"
            module.exports.getStreams = async function() {
                var mac = CryptoJS.HmacSHA256('The quick brown fox jumps over the lazy dog', 'key').toString(CryptoJS.enc.Hex);
                return [{ title: mac, url: 'https://example.com/x' }];
            };
        "#;
        let result = run_scraper(code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed[0]["title"],
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn cryptojs_aes_cbc_roundtrips_with_explicit_key_and_iv() {
        let code = r#"
            module.exports.getStreams = async function() {
                var key = CryptoJS.enc.Utf8.parse('0123456789abcdef');
                var iv = CryptoJS.enc.Utf8.parse('abcdef9876543210');
                var plaintext = 'secret stream url payload';
                var encrypted = CryptoJS.AES.encrypt(plaintext, key, { iv: iv });
                var decrypted = CryptoJS.AES.decrypt(encrypted, key, { iv: iv }).toString(CryptoJS.enc.Utf8);
                return [{ title: decrypted, url: 'https://example.com/x' }];
            };
        "#;
        let result = run_scraper(code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["title"], "secret stream url payload");
    }

    #[test]
    fn webcrypto_subtle_digest_matches_cryptojs() {
        let code = r#"
            module.exports.getStreams = async function() {
                var buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc'));
                var hex = Array.from(new Uint8Array(buf)).map(function(b) {
                    return b.toString(16).padStart(2, '0');
                }).join('');
                return [{ title: hex, url: 'https://example.com/x' }];
            };
        "#;
        let result = run_scraper(code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed[0]["title"],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn webcrypto_ecdsa_verify_accepts_a_python_produced_signature() {
        let key_hex = "3059301306072a8648ce3d020106082a8648ce3d030107034200049e6a723242258b9d87c8362fd321140b80e16d1671b5cb9b2dba3da7ccc42b82380102e3fd415ed40dfc8c4b4b218218995327daedc7eb35493f4f5b419aeaf8";
        let sig_hex = "f982b6fd52964b591329f5b503627dc1dc5b7f74ff0cf9acc840ab160636a99a526a7f11ee179e77d176827ab0035ee92653e3e7408c6c8fea3f566ec79e8c8f";
        let code = format!(
            r#"
            module.exports.getStreams = async function() {{
                function hexToBytes(hex) {{
                    var bytes = new Uint8Array(hex.length / 2);
                    for (var i = 0; i < hex.length; i += 2) bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
                    return bytes;
                }}
                var keyBytes = hexToBytes('{key_hex}');
                var sigBytes = hexToBytes('{sig_hex}');
                var key = await crypto.subtle.importKey('spki', keyBytes, {{ name: 'ECDSA', namedCurve: 'P-256' }}, false, ['verify']);
                var ok = await crypto.subtle.verify({{ name: 'ECDSA', hash: 'SHA-256' }}, key, sigBytes, new TextEncoder().encode('hello plugin signature test'));
                return [{{ title: String(ok), url: 'https://example.com/x' }}];
            }};
            "#,
            key_hex = key_hex,
            sig_hex = sig_hex,
        );
        let result = run_scraper(&code, "1", "movie");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["title"], "true");
    }

    #[test]
    fn scraper_id_and_settings_are_visible_as_globals() {
        let code = r#"
            module.exports.getStreams = async function() {
                return [{ title: SCRAPER_ID + ':' + SCRAPER_SETTINGS.quality, url: 'https://example.com/x' }];
            };
        "#;
        let result = execute_scraper(
            mock_client(),
            code.to_string(),
            "test-repository".to_string(),
            "my-scraper".to_string(),
            r#"{"quality":"1080p"}"#.to_string(),
            "1".to_string(),
            "movie".to_string(),
            None,
            None,
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["title"], "my-scraper:1080p");
    }

    #[test]
    fn settings_layout_returns_declared_field_layout() {
        let code = r#"
            module.exports.onSettings = async function() {
                return [
                    { type: 'header', label: 'General' },
                    { key: 'apiKey', type: 'text', label: 'API Key', isPassword: true },
                    {
                        key: 'quality',
                        type: 'select',
                        label: 'Preferred Quality',
                        defaultValue: '1080p',
                        options: [
                            { label: '720p', value: '720p' },
                            { label: '1080p', value: '1080p' }
                        ]
                    },
                    { key: 'enabled', type: 'toggle', label: 'Enable Feature', defaultValue: true }
                ];
            };
        "#;

        let result = get_settings_layout(code.to_string(), "my-scraper".to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 4);
        assert_eq!(parsed[1]["key"], "apiKey");
        assert_eq!(parsed[2]["options"][1]["value"], "1080p");
        assert_eq!(parsed[3]["defaultValue"], true);
    }

    #[test]
    fn settings_layout_defaults_to_empty_array_when_undefined() {
        let code = r#"
            module.exports.getStreams = async function() { return []; };
        "#;

        let result = get_settings_layout(code.to_string(), "my-scraper".to_string());
        assert_eq!(result, "[]");
    }
}
