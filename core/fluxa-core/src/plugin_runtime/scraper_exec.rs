use super::host_functions::{
    BASE64_POLYFILL, CHEERIO_POLYFILL, CRYPTO_POLYFILL, TEXT_ENCODER_POLYFILL,
    register_host_functions,
};
use super::web_compat::WEB_COMPAT_POLYFILL;
use super::{PLUGIN_TIMEOUT_SECS, PluginHttpClient, plugin_memory_limit};
use crate::plugin_runtime::dom_bridge::DomBridge;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function, Persistent, Promise};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

struct QuickJsState {
    runtime: AsyncRuntime,
    context: AsyncContext,
}

thread_local! {
    static QUICKJS_STATE: RefCell<HashMap<String, QuickJsState>> =
        RefCell::new(HashMap::new());
    static SCRAPER_CACHE: RefCell<HashMap<(String, u64), Persistent<Function<'static>>>> =
        RefCell::new(HashMap::new());
}

const MAX_QUICKJS_PLUGIN_CONTEXTS: usize = if cfg!(target_os = "android") { 2 } else { 4 };

async fn quickjs_context(scraper_id: &str) -> Result<(AsyncRuntime, AsyncContext, bool), String> {
    if let Some((runtime, context)) = QUICKJS_STATE.with(|state| {
        state
            .borrow()
            .get(scraper_id)
            .map(|state| (state.runtime.clone(), state.context.clone()))
    }) {
        return Ok((runtime, context, false));
    }

    let runtime = AsyncRuntime::new().map_err(|e| e.to_string())?;
    let context = AsyncContext::full(&runtime)
        .await
        .map_err(|e| e.to_string())?;
    QUICKJS_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.len() >= MAX_QUICKJS_PLUGIN_CONTEXTS {
            if let Some(evicted) = state.keys().next().cloned() {
                state.remove(&evicted);
                SCRAPER_CACHE.with(|cache| {
                    cache.borrow_mut().retain(|(id, _), _| id != &evicted);
                });
            }
        }
        state.insert(
            scraper_id.to_string(),
            QuickJsState {
                runtime: runtime.clone(),
                context: context.clone(),
            },
        );
    });
    Ok((runtime, context, true))
}

#[expect(
    clippy::too_many_arguments,
    reason = "FFI scraper contract has eight independently optional wire fields"
)]
pub(super) async fn run(
    client: Arc<dyn PluginHttpClient>,
    code: String,
    context_key: String,
    scraper_id: String,
    scraper_settings_json: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    let (qjs_rt, ctx, new_context) = quickjs_context(&context_key).await?;
    qjs_rt.set_memory_limit(plugin_memory_limit()).await;
    let deadline = std::time::Instant::now() + Duration::from_secs(PLUGIN_TIMEOUT_SECS);
    qjs_rt
        .set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)))
        .await;
    if new_context {
        tokio::task::spawn_local(qjs_rt.drive());
    }

    let dom = DomBridge::new();
    let mut code_hasher = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut code_hasher);
    let code_key = code_hasher.finish();

    let eval_result: rquickjs::Result<String> = ctx
        .async_with(async |ctx| {
            register_host_functions(&ctx, &dom, client)?;

            if new_context {
                let setup = format!(
                    r#"
                    globalThis.global = globalThis;
                    globalThis.window = globalThis;
                    globalThis.self = globalThis;
                    {web_compat_polyfill}
                    {base64_polyfill}
                    {text_encoder_polyfill}
                    {crypto_polyfill}
                    {cheerio_polyfill}
                    globalThis.fetch = function(url, options) {{
                        options = options || {{}};
                        var requestUrl = url && url.href ? String(url.href) : String(url);
                        var method = String(options.method || 'GET').toUpperCase();
                        var headersJson = JSON.stringify(__normalize_fetch_headers(options.headers));
                        var body = options.body === undefined || options.body === null ? null : String(options.body);
                        var followRedirects = options.redirect !== 'manual';
                        if (options.signal && options.signal.aborted) return Promise.reject(new Error('AbortError'));
                        var raw = __native_fetch(requestUrl, method, headersJson, body, followRedirects);
                        var parsed = JSON.parse(raw);
                        var response = {{
                            ok: parsed.ok,
                            status: parsed.status,
                            statusText: parsed.statusText || '',
                            url: parsed.url || requestUrl,
                            headers: new Headers(parsed.headers || {{}}),
                            text: function() {{ return Promise.resolve(parsed.body); }},
                            json: function() {{
                                try {{
                                    if (parsed.body === null || parsed.body === undefined || parsed.body === '') return Promise.resolve(null);
                                    return Promise.resolve(JSON.parse(parsed.body));
                                }} catch (e) {{ return Promise.resolve(null); }}
                            }},
                            clone: function() {{ return response; }}
                        }};
                        return Promise.resolve(response);
                    }};
                    var require = function(name) {{
                        if (name.indexOf('cheerio') !== -1) return cheerio;
                        if (name === 'crypto-js') return CryptoJS;
                        throw new Error('module not available: ' + name);
                    }};
                    "#,
                    web_compat_polyfill = WEB_COMPAT_POLYFILL,
                    base64_polyfill = BASE64_POLYFILL,
                    text_encoder_polyfill = TEXT_ENCODER_POLYFILL,
                    crypto_polyfill = CRYPTO_POLYFILL,
                    cheerio_polyfill = CHEERIO_POLYFILL,
                );
                ctx.eval::<(), _>(setup).catch(&ctx).map_err(|e| {
                    rquickjs::Error::new_from_js_message("plugin", "setup", e.to_string())
                })?;
            }

            let scraper_settings_arg = if scraper_settings_json.trim().is_empty() {
                "{}".to_string()
            } else {
                scraper_settings_json.clone()
            };
            let cache_key = (context_key.clone(), code_key);
            let function = SCRAPER_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if let Some(function) = cache.get(&cache_key) {
                    return function
                        .clone()
                        .restore(&ctx)
                        .map_err(|error| rquickjs::Error::new_from_js_message("plugin", "cache", error.to_string()));
                }

                let script = format!(
                    r#"(function() {{
                        return async function(scraperId, settingsJson, tmdbId, mediaType, season, episode) {{
                            globalThis.SCRAPER_ID = scraperId;
                            try {{
                                globalThis.SCRAPER_SETTINGS = JSON.parse(settingsJson || '{{}}');
                            }} catch (e) {{
                                globalThis.SCRAPER_SETTINGS = {{}};
                            }}
                            var module = {{ exports: {{}} }};
                            var exports = module.exports;
                            (function() {{
                                {code}
                            }})();
                            try {{
                                var getStreams = module.exports.getStreams || globalThis.getStreams;
                                if (!getStreams) return '[]';
                                var streams = await getStreams(tmdbId, mediaType, season, episode);
                                return JSON.stringify(streams || []);
                            }} catch (e) {{
                                return '[]';
                            }}
                        }};
                    }})()"#,
                    code = code,
                );
                let function: Function = ctx.eval(script).catch(&ctx).map_err(|e| {
                    rquickjs::Error::new_from_js_message("plugin", "compile", e.to_string())
                })?;
                if cache.len() >= 32 {
                    cache.clear();
                }
                cache.insert(cache_key, Persistent::save(&ctx, function.clone()));
                Ok(function)
            })?;

            let promise: Promise = function.call((
                scraper_id,
                scraper_settings_arg,
                tmdb_id,
                media_type,
                season,
                episode,
            ))?;
            let result: String = promise.into_future().await.catch(&ctx).map_err(|e| {
                rquickjs::Error::new_from_js_message("plugin", "execute", e.to_string())
            })?;
            Ok(result)
        })
        .await;

    let result = eval_result.map_err(|e| e.to_string())?;
    qjs_rt.idle().await;
    Ok(result)
}
