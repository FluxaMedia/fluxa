use super::host_functions::{
    BASE64_POLYFILL, CHEERIO_POLYFILL, CRYPTO_POLYFILL, TEXT_ENCODER_POLYFILL,
    register_host_functions,
};
use super::{
    PLUGIN_TIMEOUT_SECS, PluginHttpClient, PluginHttpRequest, PluginHttpResponse,
    plugin_memory_limit,
};
use crate::plugin_runtime::dom_bridge::DomBridge;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(super) async fn run_settings_layout(code: String, scraper_id: String) -> String {
    let qjs_rt = match AsyncRuntime::new() {
        Ok(rt) => rt,
        Err(_) => return "[]".to_string(),
    };
    qjs_rt.set_memory_limit(plugin_memory_limit()).await;
    let deadline = std::time::Instant::now() + Duration::from_secs(PLUGIN_TIMEOUT_SECS);
    qjs_rt
        .set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)))
        .await;
    tokio::task::spawn_local(qjs_rt.drive());
    let ctx = match AsyncContext::full(&qjs_rt).await {
        Ok(ctx) => ctx,
        Err(_) => return "[]".to_string(),
    };

    let captured: Arc<Mutex<Option<String>>> = Default::default();
    let captured_clone = captured.clone();
    let dom = DomBridge::new();
    let client: Arc<dyn PluginHttpClient> = Arc::new(NoopHttpClient);

    let eval_result: rquickjs::Result<()> = ctx
        .async_with(async |ctx| {
            register_host_functions(&ctx, &dom, client)?;

            let scraper_id_arg = serde_json::to_string(&scraper_id).unwrap_or_else(|_| "\"\"".into());

            let script = format!(
                r#"
                globalThis.global = globalThis;
                globalThis.window = globalThis;
                globalThis.SCRAPER_ID = {scraper_id_arg};
                globalThis.SCRAPER_SETTINGS = {{}};

                function fetch(url, options) {{
                    options = options || {{}};
                    var method = options.method || 'GET';
                    var headersJson = JSON.stringify(options.headers || {{}});
                    var body = options.body === undefined || options.body === null ? null : String(options.body);
                    var followRedirects = options.redirect !== 'manual';
                    var raw = __native_fetch(url, method, headersJson, body, followRedirects);
                    var parsed = JSON.parse(raw);
                    return Promise.resolve({{
                        ok: parsed.ok,
                        status: parsed.status,
                        text: function() {{ return Promise.resolve(parsed.body); }},
                        json: function() {{
                            try {{ return Promise.resolve(JSON.parse(parsed.body)); }}
                            catch (e) {{ return Promise.resolve(null); }}
                        }}
                    }});
                }}

                {base64_polyfill}
                {text_encoder_polyfill}
                {crypto_polyfill}
                {cheerio_polyfill}

                var require = function(name) {{
                    if (name.indexOf('cheerio') !== -1) return cheerio;
                    if (name === 'crypto-js') return CryptoJS;
                    throw new Error('module not available: ' + name);
                }};

                var module = {{ exports: {{}} }};
                var exports = module.exports;
                (function() {{
                    {code}
                }})();

                (async function() {{
                    try {{
                        var onSettings = module.exports.onSettings || globalThis.onSettings;
                        if (!onSettings) {{
                            __capture_result(JSON.stringify([]));
                            return;
                        }}
                        var layout = await onSettings();
                        __capture_result(JSON.stringify(layout || []));
                    }} catch (e) {{
                        __capture_result(JSON.stringify([]));
                    }}
                }})();
                "#,
                base64_polyfill = BASE64_POLYFILL,
                text_encoder_polyfill = TEXT_ENCODER_POLYFILL,
                crypto_polyfill = CRYPTO_POLYFILL,
                cheerio_polyfill = CHEERIO_POLYFILL,
                code = code,
            );

            ctx.globals().set(
                "__capture_result",
                Function::new(ctx.clone(), move |s: String| {
                    if let Ok(mut captured) = captured_clone.lock() {
                        *captured = Some(s);
                    }
                })?,
            )?;

            ctx.eval::<(), _>(script).catch(&ctx).map_err(|e| {
                rquickjs::Error::new_from_js_message("plugin", "js", e.to_string())
            })?;

            Ok(())
        })
        .await;

    if eval_result.is_err() {
        return "[]".to_string();
    }
    qjs_rt.idle().await;

    captured
        .lock()
        .ok()
        .and_then(|mut captured| captured.take())
        .unwrap_or_else(|| "[]".to_string())
}

struct NoopHttpClient;

impl PluginHttpClient for NoopHttpClient {
    fn fetch(&self, _request: PluginHttpRequest) -> PluginHttpResponse {
        PluginHttpResponse {
            status: 0,
            headers: HashMap::new(),
            body: String::new(),
            ok: false,
            error: Some("network disabled in settings-layout evaluation".to_string()),
        }
    }
}
