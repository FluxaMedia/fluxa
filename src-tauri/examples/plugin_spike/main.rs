mod dom_bridge;

use dom_bridge::DomBridge;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function};
use std::sync::Arc;
use std::time::Duration;

const PLUGIN_CODE: &str = include_str!("../plugin_spike_fixtures/moviesdrive.js");

fn console_log(msg: String) {
    println!("[plugin] {msg}");
}

async fn native_fetch(url: String) -> rquickjs::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| rquickjs::Error::new_from_js_message("fetch", "reqwest", e.to_string()))?;

    match client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Ok(format!(
                "{{\"ok\":{},\"status\":{},\"body\":{}}}",
                status < 400,
                status,
                serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".into())
            ))
        }
        Err(e) => Ok(format!(
            "{{\"ok\":false,\"status\":0,\"body\":\"\",\"error\":{}}}",
            serde_json::to_string(&e.to_string()).unwrap_or_else(|_| "\"\"".into())
        )),
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    let local = tokio::task::LocalSet::new();

    if std::env::args().nth(1).as_deref() == Some("--dom-test") {
        local.block_on(&rt, run_dom_test());
    } else {
        local.block_on(&rt, run());
    }
}

/// Exercises the cheerio bridge directly against static HTML, independent
/// of network access, since a real scraper run may never reach the HTML
/// parsing path if the site lookup fails first.
async fn run_dom_test() {
    let rt = AsyncRuntime::new().expect("create quickjs runtime");
    tokio::task::spawn_local(rt.drive());
    let ctx = AsyncContext::full(&rt)
        .await
        .expect("create quickjs context");
    let dom = DomBridge::new();

    let result: rquickjs::Result<String> = ctx
        .async_with(async |ctx| {
            register_host_functions(&ctx, &dom)?;
            let script = format!(
                r#"
                {cheerio_polyfill}

                var html = `
                    <html><body>
                        <ul class="items">
                            <li class="row" data-quality="1080p"><a href="/a">Alpha</a></li>
                            <li class="row" data-quality="720p"><a href="/b">Beta Download</a></li>
                            <li class="row" data-quality="480p"><a href="/c">Gamma</a></li>
                        </ul>
                    </body></html>
                `;
                var $ = cheerio.load(html);
                var rows = $('.row');
                var out = [];
                rows.each(function(i, el) {{
                    out.push({{
                        quality: el.attr('data-quality'),
                        text: el.text().trim(),
                        href: el.find('a').attr('href'),
                    }});
                }});
                out.push({{ containsCheck: $('.row:contains("Beta")').length }});
                out.push({{ secondViaNext: $('.row').first().next().text().trim() }});
                out.push({{ firstViaPrev: $('.row').last().prev().prev().text().trim() }});
                JSON.stringify(out);
                "#,
                cheerio_polyfill = CHEERIO_POLYFILL,
            );
            ctx.eval::<String, _>(script).catch(&ctx).map_err(|e| {
                eprintln!("JS error: {e}");
                rquickjs::Error::Unknown
            })
        })
        .await;

    match result {
        Ok(json) => println!("=== dom-test result ===\n{json}"),
        Err(e) => eprintln!("dom-test failed: {e}"),
    }
}

async fn run() {
    let tmdb_id = std::env::args().nth(1).unwrap_or_else(|| "27205".into());

    let rt = AsyncRuntime::new().expect("create quickjs runtime");
    tokio::task::spawn_local(rt.drive());
    let ctx = AsyncContext::full(&rt)
        .await
        .expect("create quickjs context");

    let captured: std::sync::Arc<std::sync::Mutex<Option<String>>> = Default::default();
    let captured_clone = captured.clone();
    let dom = DomBridge::new();

    let eval_result: rquickjs::Result<()> = ctx.async_with(async |ctx| {
        register_host_functions(&ctx, &dom)?;

        let polyfill = format!(
            r#"
            globalThis.global = globalThis;
            globalThis.window = globalThis;

            function fetch(url, options) {{
                options = options || {{}};
                return __native_fetch(url).then(function(raw) {{
                    var parsed = JSON.parse(raw);
                    return {{
                        ok: parsed.ok,
                        status: parsed.status,
                        text: function() {{ return Promise.resolve(parsed.body); }},
                        json: function() {{
                            try {{ return Promise.resolve(JSON.parse(parsed.body)); }}
                            catch (e) {{ return Promise.resolve(null); }}
                        }}
                    }};
                }});
            }}

            {cheerio_polyfill}

            var require = function(name) {{
                if (name.indexOf('cheerio') !== -1) return cheerio;
                throw new Error('module not available in spike: ' + name);
            }};

            var module = {{ exports: {{}} }};
            var exports = module.exports;
            (function() {{
                {code}
            }})();

            (async function() {{
                try {{
                    var streams = await module.exports.getStreams("{tmdb_id}", "movie", undefined, undefined);
                    __capture_result(JSON.stringify(streams || []));
                }} catch (e) {{
                    __capture_result(JSON.stringify({{ error: String(e && e.message ? e.message : e) }}));
                }}
            }})();
            "#,
            cheerio_polyfill = CHEERIO_POLYFILL,
            code = PLUGIN_CODE,
            tmdb_id = tmdb_id,
        );

        ctx.globals().set(
            "__capture_result",
            Function::new(ctx.clone(), move |s: String| {
                *captured_clone.lock().expect("capture lock poisoned") = Some(s);
            })?,
        )?;

        ctx.eval::<(), _>(polyfill).catch(&ctx).map_err(|e| {
            eprintln!("JS error: {e}");
            rquickjs::Error::Unknown
        })?;

        Ok(())
    })
    .await;

    if let Err(e) = eval_result {
        eprintln!("spike failed: {e}");
        return;
    }

    rt.idle().await;

    let final_value = captured.lock().expect("capture lock poisoned").take();
    match final_value {
        Some(json) => println!("\n=== getStreams result ===\n{json}"),
        None => eprintln!("spike failed: no result captured"),
    }
}

fn register_host_functions(ctx: &Ctx<'_>, dom: &Arc<DomBridge>) -> rquickjs::Result<()> {
    ctx.globals().set(
        "console",
        rquickjs::Object::new(ctx.clone()).and_then(|obj| {
            obj.set("log", Function::new(ctx.clone(), console_log)?)?;
            obj.set(
                "warn",
                Function::new(ctx.clone(), |msg: String| println!("[plugin:warn] {msg}"))?,
            )?;
            obj.set(
                "error",
                Function::new(ctx.clone(), |msg: String| println!("[plugin:error] {msg}"))?,
            )?;
            Ok(obj)
        })?,
    )?;

    ctx.globals().set(
        "__native_fetch",
        Function::new(ctx.clone(), rquickjs::function::Async(native_fetch))?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_load",
        Function::new(ctx.clone(), move |html: String| d.load(html))?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_select",
        Function::new(ctx.clone(), move |doc_id: String, selector: String| {
            d.select(doc_id, selector)
        })?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_find",
        Function::new(
            ctx.clone(),
            move |doc_id: String, element_id: String, selector: String| {
                d.find(doc_id, element_id, selector)
            },
        )?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_text",
        Function::new(ctx.clone(), move |_doc_id: String, element_ids: String| {
            d.text(element_ids)
        })?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_html",
        Function::new(ctx.clone(), move |doc_id: String, element_id: String| {
            d.html(doc_id, element_id)
        })?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_inner_html",
        Function::new(ctx.clone(), move |_doc_id: String, element_id: String| {
            d.inner_html(element_id)
        })?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_attr",
        Function::new(
            ctx.clone(),
            move |_doc_id: String, element_id: String, attr_name: String| {
                d.attr(element_id, attr_name)
            },
        )?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_next",
        Function::new(ctx.clone(), move |doc_id: String, element_id: String| {
            d.next(doc_id, element_id)
        })?,
    )?;

    let d = dom.clone();
    ctx.globals().set(
        "__cheerio_prev",
        Function::new(ctx.clone(), move |doc_id: String, element_id: String| {
            d.prev(doc_id, element_id)
        })?,
    )?;

    Ok(())
}

/// Ported from Nuvio's `JsBindings.buildPolyfillCode` cheerio wrapper — a
/// jQuery-like `$` over the id-cache bridge above, so plugins written for
/// Nuvio's `require('cheerio')` run here unmodified.
const CHEERIO_POLYFILL: &str = r#"
var cheerio = {
    load: function(html) {
        var docId = __cheerio_load(html);
        var $ = function(selector, context) {
            if (selector && selector._elementIds) return selector;
            if (context && context._elementIds && context._elementIds.length > 0) {
                var allIds = [];
                for (var i = 0; i < context._elementIds.length; i++) {
                    var childIdsJson = __cheerio_find(docId, context._elementIds[i], selector);
                    var childIds = JSON.parse(childIdsJson);
                    allIds = allIds.concat(childIds);
                }
                return createCheerioWrapperFromIds(docId, allIds);
            }
            return createCheerioWrapper(docId, selector);
        };
        $.html = function(el) {
            if (el && el._elementIds && el._elementIds.length > 0) {
                return __cheerio_html(docId, el._elementIds[0]);
            }
            return __cheerio_html(docId, '');
        };
        return $;
    }
};

function createCheerioWrapper(docId, selector) {
    var elementIds;
    if (typeof selector === 'string') {
        var idsJson = __cheerio_select(docId, selector);
        elementIds = JSON.parse(idsJson);
    } else {
        elementIds = [];
    }
    return createCheerioWrapperFromIds(docId, elementIds);
}

function createCheerioWrapperFromIds(docId, ids) {
    var wrapper = {
        _docId: docId,
        _elementIds: ids,
        length: ids.length,
        each: function(callback) {
            for (var i = 0; i < ids.length; i++) {
                var elWrapper = createCheerioWrapperFromIds(docId, [ids[i]]);
                callback.call(elWrapper, i, elWrapper);
            }
            return wrapper;
        },
        find: function(sel) {
            var allIds = [];
            for (var i = 0; i < ids.length; i++) {
                var childIdsJson = __cheerio_find(docId, ids[i], sel);
                var childIds = JSON.parse(childIdsJson);
                allIds = allIds.concat(childIds);
            }
            return createCheerioWrapperFromIds(docId, allIds);
        },
        text: function() {
            if (ids.length === 0) return '';
            return __cheerio_text(docId, ids.join(','));
        },
        html: function() {
            if (ids.length === 0) return '';
            return __cheerio_inner_html(docId, ids[0]);
        },
        attr: function(name) {
            if (ids.length === 0) return undefined;
            var val = __cheerio_attr(docId, ids[0], name);
            return val === '__UNDEFINED__' ? undefined : val;
        },
        first: function() { return createCheerioWrapperFromIds(docId, ids.length > 0 ? [ids[0]] : []); },
        last: function() { return createCheerioWrapperFromIds(docId, ids.length > 0 ? [ids[ids.length - 1]] : []); },
        next: function() {
            var nextIds = [];
            for (var i = 0; i < ids.length; i++) {
                var nextId = __cheerio_next(docId, ids[i]);
                if (nextId && nextId !== '__NONE__') nextIds.push(nextId);
            }
            return createCheerioWrapperFromIds(docId, nextIds);
        },
        prev: function() {
            var prevIds = [];
            for (var i = 0; i < ids.length; i++) {
                var prevId = __cheerio_prev(docId, ids[i]);
                if (prevId && prevId !== '__NONE__') prevIds.push(prevId);
            }
            return createCheerioWrapperFromIds(docId, prevIds);
        },
        eq: function(index) {
            if (index >= 0 && index < ids.length) return createCheerioWrapperFromIds(docId, [ids[index]]);
            return createCheerioWrapperFromIds(docId, []);
        },
        get: function(index) {
            if (typeof index === 'number') {
                if (index >= 0 && index < ids.length) return createCheerioWrapperFromIds(docId, [ids[index]]);
                return undefined;
            }
            return ids.map(function(id) { return createCheerioWrapperFromIds(docId, [id]); });
        },
        map: function(callback) {
            var results = [];
            for (var i = 0; i < ids.length; i++) {
                var elWrapper = createCheerioWrapperFromIds(docId, [ids[i]]);
                var result = callback.call(elWrapper, i, elWrapper);
                if (result !== undefined && result !== null) results.push(result);
            }
            return {
                length: results.length,
                get: function(index) { return typeof index === 'number' ? results[index] : results; },
                toArray: function() { return results; }
            };
        },
        filter: function(selectorOrCallback) {
            if (typeof selectorOrCallback === 'function') {
                var filteredIds = [];
                for (var i = 0; i < ids.length; i++) {
                    var elWrapper = createCheerioWrapperFromIds(docId, [ids[i]]);
                    var result = selectorOrCallback.call(elWrapper, i, elWrapper);
                    if (result) filteredIds.push(ids[i]);
                }
                return createCheerioWrapperFromIds(docId, filteredIds);
            }
            return wrapper;
        },
        children: function(sel) { return this.find(sel || '*'); },
        parent: function() { return createCheerioWrapperFromIds(docId, []); },
        toArray: function() { return ids.map(function(id) { return createCheerioWrapperFromIds(docId, [id]); }); }
    };
    return wrapper;
}
"#;
