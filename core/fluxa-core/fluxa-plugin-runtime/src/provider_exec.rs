use super::host_functions::register_host_functions;
use super::scraper_exec::{quickjs_context, setup_script};
use super::{PLUGIN_TIMEOUT_SECS, PluginHttpClient, plugin_memory_limit};
use crate::dom_bridge::DomBridge;
use rquickjs::{CatchResultExt, Promise};
use std::sync::Arc;
use std::time::Duration;

const SEANIME_SHIMS: &str = include_str!("polyfills/seanime.js");

pub(super) async fn run(
    client: Arc<dyn PluginHttpClient>,
    code: String,
    context_key: String,
    settings_json: String,
    method: String,
    args_json: String,
) -> Result<String, String> {
    let (runtime, context, fresh) = quickjs_context(&context_key).await?;
    runtime.set_memory_limit(plugin_memory_limit()).await;
    runtime.set_max_stack_size(crate::PLUGIN_STACK_LIMIT).await;
    let deadline = std::time::Instant::now() + Duration::from_secs(PLUGIN_TIMEOUT_SECS);
    runtime
        .set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)))
        .await;
    if fresh {
        tokio::task::spawn_local(runtime.drive());
    }
    let dom = DomBridge::new();
    let outcome: rquickjs::Result<String> = context
        .async_with(async |ctx| {
            register_host_functions(&ctx, &dom, client)?;
            let js_error = |stage: &'static str| {
                move |error: String| rquickjs::Error::new_from_js_message("provider", stage, error)
            };
            if fresh {
                let setup = format!("{}\n{}", setup_script(), SEANIME_SHIMS);
                ctx.eval::<(), _>(setup)
                    .catch(&ctx)
                    .map_err(|e| js_error("setup")(e.to_string()))?;
            }
            let settings = if settings_json.trim().is_empty() {
                "{}"
            } else {
                settings_json.as_str()
            };
            let literal = |text: &str| serde_json::to_string(text).unwrap_or_default();
            let script = format!(
                r#"(async function() {{
                    try {{ globalThis.SCRAPER_SETTINGS = JSON.parse({settings}); }} catch (e) {{ globalThis.SCRAPER_SETTINGS = {{}}; }}
                    var module = {{ exports: {{}} }};
                    var exports = module.exports;
                    var ProviderClass = (function() {{
                        {code}
                        return typeof Provider !== 'undefined' ? Provider : globalThis.Provider;
                    }})();
                    if (!ProviderClass) throw new Error('extension has no Provider class');
                    var instance = new ProviderClass();
                    var args = JSON.parse({args});
                    var result = await instance[{method}].apply(instance, args);
                    return JSON.stringify(result === undefined ? null : result);
                }})()"#,
                settings = literal(settings),
                code = code,
                args = literal(&args_json),
                method = literal(&method),
            );
            let promise: Promise = ctx
                .eval(script)
                .catch(&ctx)
                .map_err(|e| js_error("compile")(e.to_string()))?;
            promise
                .into_future::<String>()
                .await
                .catch(&ctx)
                .map_err(|e| js_error("execute")(e.to_string()))
        })
        .await;
    let result = outcome.map_err(|error| error.to_string())?;
    runtime.idle().await;
    Ok(result)
}
