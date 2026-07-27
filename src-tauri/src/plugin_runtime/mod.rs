mod crypto_bridge;
mod dom_bridge;

use crate::net_guard;
use dom_bridge::DomBridge;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function};
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const PLUGIN_TIMEOUT_SECS: u64 = 60;
const FETCH_TIMEOUT_SECS: u64 = 15;
const PLUGIN_MEMORY_LIMIT: usize = 256 * 1024 * 1024;

/// Runs a Nuvio-compatible scraper plugin's `getStreams()` inside a
/// sandboxed QuickJS VM and returns its raw (unvalidated) JSON output.
/// Must be called from a blocking context (e.g. `spawn_blocking`) — it
/// builds its own single-threaded tokio runtime because the JS engine's
/// internal state (via `scraper`/`tendril`) isn't `Send`.
pub fn execute_scraper(
    code: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, async move {
        tokio::time::timeout(
            Duration::from_secs(PLUGIN_TIMEOUT_SECS),
            run(code, tmdb_id, media_type, season, episode),
        )
        .await
        .unwrap_or_else(|_| Err("plugin timed out".to_string()))
    })
}

async fn run(
    code: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    let qjs_rt = AsyncRuntime::new().map_err(|e| e.to_string())?;
    qjs_rt.set_memory_limit(PLUGIN_MEMORY_LIMIT).await;
    let deadline = std::time::Instant::now() + Duration::from_secs(PLUGIN_TIMEOUT_SECS);
    qjs_rt
        .set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)))
        .await;
    tokio::task::spawn_local(qjs_rt.drive());
    let ctx = AsyncContext::full(&qjs_rt)
        .await
        .map_err(|e| e.to_string())?;

    let captured: Arc<Mutex<Option<String>>> = Default::default();
    let captured_clone = captured.clone();
    let dom = DomBridge::new();

    let eval_result: rquickjs::Result<()> = ctx
        .async_with(async |ctx| {
            register_host_functions(&ctx, &dom)?;

            let tmdb_id_arg = serde_json::to_string(&tmdb_id).unwrap_or_else(|_| "\"\"".into());
            let media_type_arg =
                serde_json::to_string(&media_type).unwrap_or_else(|_| "\"movie\"".into());
            let season_arg = season.map(|s| s.to_string()).unwrap_or_else(|| "undefined".into());
            let episode_arg = episode.map(|e| e.to_string()).unwrap_or_else(|| "undefined".into());

            let script = format!(
                r#"
                globalThis.global = globalThis;
                globalThis.window = globalThis;

                function fetch(url, options) {{
                    options = options || {{}};
                    return __native_fetch(url, JSON.stringify({{
                        method: options.method,
                        headers: options.headers,
                        body: options.body
                    }})).then(function(raw) {{
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
                        var getStreams = module.exports.getStreams || globalThis.getStreams;
                        if (!getStreams) {{
                            __capture_result(JSON.stringify([]));
                            return;
                        }}
                        var streams = await getStreams({tmdb_id_arg}, {media_type_arg}, {season_arg}, {episode_arg});
                        __capture_result(JSON.stringify(streams || []));
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
                    *captured_clone.lock().expect("capture lock poisoned") = Some(s);
                })?,
            )?;

            ctx.eval::<(), _>(script).catch(&ctx).map_err(|e| {
                rquickjs::Error::new_from_js_message("plugin", "js", e.to_string())
            })?;

            Ok(())
        })
        .await;

    eval_result.map_err(|e| e.to_string())?;
    qjs_rt.idle().await;

    let result = captured
        .lock()
        .expect("capture lock poisoned")
        .take()
        .unwrap_or_else(|| "[]".to_string());
    Ok(result)
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginFetchOptions {
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

async fn native_fetch(url: String, options_json: String) -> rquickjs::Result<String> {
    let client = match net_guard::vetted_client(&url, Duration::from_secs(FETCH_TIMEOUT_SECS)).await
    {
        Ok(client) => client,
        Err(message) => return Ok(fetch_error_json(&url, &message)),
    };

    let options = serde_json::from_str::<PluginFetchOptions>(&options_json).unwrap_or_default();
    let method = options
        .method
        .as_deref()
        .and_then(|value| reqwest::Method::from_bytes(value.as_bytes()).ok())
        .unwrap_or(reqwest::Method::GET);
    let mut request = client.request(method, &url);
    if let Some(headers) = options.headers {
        for (name, value) in headers {
            if let (Ok(name), Ok(value)) = (
                reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                request = request.header(name, value);
            }
        }
    }
    if let Some(body) = options.body {
        request = request.body(body);
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            Ok(format!(
                "{{\"ok\":{},\"status\":{},\"body\":{}}}",
                (200..300).contains(&status),
                status,
                serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".into())
            ))
        }
        Err(e) => Ok(fetch_error_json(&url, &e.to_string())),
    }
}

fn fetch_error_json(url: &str, message: &str) -> String {
    format!(
        "{{\"ok\":false,\"status\":0,\"body\":\"\",\"url\":{},\"error\":{}}}",
        serde_json::to_string(url).unwrap_or_else(|_| "\"\"".into()),
        serde_json::to_string(message).unwrap_or_else(|_| "\"\"".into())
    )
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(hex: &str) -> Vec<u8> {
    let clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    (0..clean.len() / 2)
        .filter_map(|i| u8::from_str_radix(clean.get(i * 2..i * 2 + 2)?, 16).ok())
        .collect()
}

fn register_host_functions(ctx: &Ctx<'_>, dom: &Rc<DomBridge>) -> rquickjs::Result<()> {
    ctx.globals().set(
        "console",
        rquickjs::Object::new(ctx.clone()).and_then(|obj| {
            obj.set(
                "log",
                Function::new(ctx.clone(), |msg: String| log::debug!("[plugin] {msg}"))?,
            )?;
            obj.set(
                "warn",
                Function::new(ctx.clone(), |msg: String| log::warn!("[plugin] {msg}"))?,
            )?;
            obj.set(
                "error",
                Function::new(ctx.clone(), |msg: String| log::warn!("[plugin] {msg}"))?,
            )?;
            obj.set(
                "info",
                Function::new(ctx.clone(), |msg: String| log::debug!("[plugin] {msg}"))?,
            )?;
            obj.set(
                "debug",
                Function::new(ctx.clone(), |msg: String| log::debug!("[plugin] {msg}"))?,
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

    ctx.globals().set(
        "__crypto_get_random_values_hex",
        Function::new(ctx.clone(), |len: usize| {
            to_hex(&crypto_bridge::random_bytes(len))
        })?,
    )?;

    ctx.globals().set(
        "__crypto_digest_hex_raw",
        Function::new(ctx.clone(), |algorithm: String, data_hex: String| {
            crypto_bridge::digest(&algorithm, &from_hex(&data_hex))
                .map(|bytes| to_hex(&bytes))
                .unwrap_or_default()
        })?,
    )?;

    ctx.globals().set(
        "__crypto_hmac_hex_raw",
        Function::new(
            ctx.clone(),
            |algorithm: String, key_hex: String, data_hex: String| {
                crypto_bridge::hmac(&algorithm, &from_hex(&key_hex), &from_hex(&data_hex))
                    .map(|bytes| to_hex(&bytes))
                    .unwrap_or_default()
            },
        )?,
    )?;

    ctx.globals().set(
        "__crypto_pbkdf2_hex",
        Function::new(
            ctx.clone(),
            |password_hex: String,
             salt_hex: String,
             iterations: u32,
             key_size_bits: u32,
             algorithm: String| {
                crypto_bridge::pbkdf2(
                    &from_hex(&password_hex),
                    &from_hex(&salt_hex),
                    iterations,
                    key_size_bits,
                    &algorithm,
                )
                .map(|bytes| to_hex(&bytes))
                .unwrap_or_default()
            },
        )?,
    )?;

    ctx.globals().set(
        "__crypto_aes_encrypt_hex",
        Function::new(
            ctx.clone(),
            |mode: String, key_hex: String, iv_hex: String, data_hex: String| {
                crypto_bridge::aes_encrypt(
                    &mode,
                    &from_hex(&key_hex),
                    &from_hex(&iv_hex),
                    &from_hex(&data_hex),
                )
                .map(|bytes| to_hex(&bytes))
                .unwrap_or_default()
            },
        )?,
    )?;

    ctx.globals().set(
        "__crypto_aes_decrypt_hex",
        Function::new(
            ctx.clone(),
            |mode: String, key_hex: String, iv_hex: String, data_hex: String| {
                crypto_bridge::aes_decrypt(
                    &mode,
                    &from_hex(&key_hex),
                    &from_hex(&iv_hex),
                    &from_hex(&data_hex),
                )
                .map(|bytes| to_hex(&bytes))
                .unwrap_or_default()
            },
        )?,
    )?;

    ctx.globals().set(
        "__crypto_utf8_to_hex",
        Function::new(ctx.clone(), |text: String| to_hex(text.as_bytes()))?,
    )?;

    ctx.globals().set(
        "__crypto_hex_to_utf8",
        Function::new(ctx.clone(), |hex: String| {
            String::from_utf8_lossy(&from_hex(&hex)).into_owned()
        })?,
    )?;

    ctx.globals().set(
        "__crypto_sign_hex",
        Function::new(
            ctx.clone(),
            |algorithm: String, key_hex: String, data_hex: String| {
                crypto_bridge::sign(&algorithm, &from_hex(&key_hex), &from_hex(&data_hex))
                    .map(|bytes| to_hex(&bytes))
                    .unwrap_or_default()
            },
        )?,
    )?;

    ctx.globals().set(
        "__crypto_verify_hex",
        Function::new(
            ctx.clone(),
            |algorithm: String, key_hex: String, signature_hex: String, data_hex: String| {
                crypto_bridge::verify(
                    &algorithm,
                    &from_hex(&key_hex),
                    &from_hex(&signature_hex),
                    &from_hex(&data_hex),
                )
                .unwrap_or(false)
            },
        )?,
    )?;

    Ok(())
}

const BASE64_POLYFILL: &str = r#"
if (typeof atob === 'undefined') {
    globalThis.atob = function(input) {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var str = String(input).replace(/=+$/, '');
        if (str.length % 4 === 1) throw new Error('InvalidCharacterError');
        var output = '';
        var bc = 0, bs, buffer, idx = 0;
        while ((buffer = str.charAt(idx++))) {
            buffer = chars.indexOf(buffer);
            if (buffer === -1) continue;
            bs = bc % 4 ? bs * 64 + buffer : buffer;
            if (bc++ % 4) output += String.fromCharCode(255 & (bs >> ((-2 * bc) & 6)));
        }
        return output;
    };
}

if (typeof btoa === 'undefined') {
    globalThis.btoa = function(input) {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var str = String(input);
        var output = '';
        for (var block, charCode, idx = 0, map = chars;
             str.charAt(idx | 0) || (map = '=', idx % 1);
             output += map.charAt(63 & (block >> (8 - (idx % 1) * 8)))) {
            charCode = str.charCodeAt(idx += 3 / 4);
            if (charCode > 0xFF) throw new Error('InvalidCharacterError');
            block = (block << 8) | charCode;
        }
        return output;
    };
}
"#;

const TEXT_ENCODER_POLYFILL: &str = r#"
if (typeof TextEncoder === 'undefined') {
    globalThis.TextEncoder = function() {};
    TextEncoder.prototype.encode = function(str) {
        var hex = __crypto_utf8_to_hex(str);
        var bytes = new Uint8Array(hex.length / 2);
        for (var i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
        }
        return bytes;
    };
}
if (typeof TextDecoder === 'undefined') {
    globalThis.TextDecoder = function() {};
    TextDecoder.prototype.decode = function(data) {
        var bytes = data;
        if (data instanceof ArrayBuffer) bytes = new Uint8Array(data);
        var hex = '';
        for (var i = 0; i < bytes.length; i++) {
            hex += bytes[i].toString(16).padStart(2, '0');
        }
        return __crypto_hex_to_utf8(hex);
    };
}
"#;

/// A CryptoJS-compatible shim plus a `crypto.subtle`/WebCrypto shim, both
/// backed by the native `__crypto_*_hex` bridges (digest/HMAC/PBKDF2/AES),
/// ported from Nuvio's `JsBindings.buildPolyfillCode`. RSA/ECDSA sign/verify
/// are deliberately not implemented — `crypto.subtle.sign`/`verify` throw
/// clearly instead of silently no-op'ing.
const CRYPTO_POLYFILL: &str = r#"
var WordArray = {
    init: function(words, sigBytes) {
        this.words = words || [];
        this.sigBytes = sigBytes != undefined ? sigBytes : this.words.length * 4;
    },
    toString: function(encoder) {
        return (encoder || CryptoJS.enc.Hex).stringify(this);
    },
    concat: function(wordArray) {
        var thisWords = this.words;
        var thatWords = wordArray.words;
        var thisSigBytes = this.sigBytes;
        var thatSigBytes = wordArray.sigBytes;

        this.clamp();

        for (var i = 0; i < thatSigBytes; i++) {
            var thatByte = (thatWords[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
            thisWords[(thisSigBytes + i) >>> 2] |= thatByte << (24 - ((thisSigBytes + i) % 4) * 8);
        }
        this.sigBytes += thatSigBytes;
        return this;
    },
    clamp: function() {
        var words = this.words;
        var sigBytes = this.sigBytes;
        if (sigBytes % 4) {
            words[sigBytes >>> 2] &= 0xffffffff << (32 - (sigBytes % 4) * 8);
        }
        words.length = Math.ceil(sigBytes / 4);
        return this;
    },
    clone: function() {
        return __wordArrayCreate(this.words.slice(0), this.sigBytes);
    }
};

function __wordArrayCreate(words, sigBytes) {
    var wa = Object.create(WordArray);
    wa.init(words, sigBytes);
    return wa;
}

function __isWordArray(value) {
    return value && typeof value === 'object' && Array.isArray(value.words) && typeof value.sigBytes === 'number';
}

function __copyUint8Array(bytes) {
    bytes = __toUint8Array(bytes);
    var copy = new Uint8Array(bytes.length);
    copy.set(bytes);
    return copy;
}

function __toUint8Array(data) {
    if (!data) return new Uint8Array(0);
    if (data instanceof Uint8Array) return data;
    if (data instanceof ArrayBuffer) return new Uint8Array(data);
    if (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView && ArrayBuffer.isView(data)) {
        return new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength);
    }
    if (Array.isArray(data)) return new Uint8Array(data);
    if (typeof data.length === 'number') return new Uint8Array(Array.prototype.slice.call(data));
    return new Uint8Array(0);
}

function __bytesToArrayBuffer(bytes) {
    return __copyUint8Array(bytes).buffer;
}

function __wordArrayToBytes(wordArray) {
    if (!__isWordArray(wordArray)) return typeof wordArray === 'string' ? new TextEncoder().encode(wordArray) : __toUint8Array(wordArray);
    var bytes = new Uint8Array(wordArray.sigBytes);
    for (var i = 0; i < wordArray.sigBytes; i++) {
        bytes[i] = (wordArray.words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
    }
    return bytes;
}

function __bytesToWordArray(bytes) {
    bytes = __toUint8Array(bytes);
    var words = [];
    for (var i = 0; i < bytes.length; i++) {
        words[i >>> 2] |= (bytes[i] & 0xff) << (24 - (i % 4) * 8);
    }
    return __wordArrayCreate(words, bytes.length);
}

function __normalizeWordArrayInput(value) {
    if (__isWordArray(value)) return __wordArrayToBytes(value);
    if (typeof value === 'string') return new TextEncoder().encode(value);
    return __toUint8Array(value);
}

function __bytesToHex(bytes) {
    bytes = __toUint8Array(bytes);
    var out = [];
    for (var i = 0; i < bytes.length; i++) {
        var hex = bytes[i].toString(16);
        out.push(hex.length < 2 ? '0' + hex : hex);
    }
    return out.join('');
}

function __hexToBytes(hex) {
    hex = String(hex || '').replace(/[^0-9a-fA-F]/g, '');
    if (hex.length % 2) hex = '0' + hex;
    var bytes = new Uint8Array(hex.length / 2);
    for (var i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16) & 0xff;
    }
    return bytes;
}

function __concatBytes() {
    var total = 0;
    var parts = [];
    for (var i = 0; i < arguments.length; i++) {
        var part = __toUint8Array(arguments[i]);
        parts.push(part);
        total += part.length;
    }
    var out = new Uint8Array(total);
    var offset = 0;
    for (var j = 0; j < parts.length; j++) {
        out.set(parts[j], offset);
        offset += parts[j].length;
    }
    return out;
}

function __normalizeHashName(hash) {
    var name = hash && hash.name ? hash.name : hash;
    name = String(name || 'SHA-256').toUpperCase().replace(/[^A-Z0-9]/g, '');
    if (name === 'SHA1' || name === 'SHA256' || name === 'SHA384' || name === 'SHA512' || name === 'MD5') return name;
    throw new Error('Unsupported hash algorithm: ' + name);
}

function __normalizeAlgorithmName(algo) {
    var name = algo && algo.name ? algo.name : algo;
    name = String(name || '').toUpperCase();
    if (name.indexOf('AES-GCM') >= 0) return 'AES-GCM';
    if (name.indexOf('AES-CBC') >= 0) return 'AES-CBC';
    if (name.indexOf('AES-ECB') >= 0 || name === 'ECB') return 'AES-ECB';
    if (name.indexOf('PBKDF2') >= 0) return 'PBKDF2';
    if (name.indexOf('HMAC') >= 0) return 'HMAC';
    if (name.indexOf('RSASSA-PKCS1') >= 0) return 'RSASSA-PKCS1-V1_5';
    if (name.indexOf('ECDSA') >= 0) return 'ECDSA';
    return name;
}

function __aesModeName(mode, padding) {
    var normalized = __normalizeAlgorithmName(mode || 'AES-CBC');
    if (padding === CryptoJS.pad.NoPadding || padding === 'NoPadding') normalized += '-NoPadding';
    return normalized;
}

function __nativeDigestBytes(hash, dataBytes) {
    if (typeof __crypto_digest_hex_raw === 'undefined') throw new Error('Native digest bridge is unavailable');
    return __hexToBytes(__crypto_digest_hex_raw(__normalizeHashName(hash), __bytesToHex(dataBytes)));
}

function __nativeHmacBytes(hash, keyBytes, dataBytes) {
    if (typeof __crypto_hmac_hex_raw === 'undefined') throw new Error('Native HMAC bridge is unavailable');
    return __hexToBytes(__crypto_hmac_hex_raw(__normalizeHashName(hash), __bytesToHex(keyBytes), __bytesToHex(dataBytes)));
}

function __nativePbkdf2Bytes(passwordBytes, saltBytes, iterations, keySizeBits, hash) {
    if (typeof __crypto_pbkdf2_hex === 'undefined') throw new Error('Native PBKDF2 bridge is unavailable');
    return __hexToBytes(__crypto_pbkdf2_hex(__bytesToHex(passwordBytes), __bytesToHex(saltBytes), iterations, keySizeBits, __normalizeHashName(hash)));
}

function __nativeAesBytes(encrypt, mode, keyBytes, ivBytes, dataBytes) {
    var fn = encrypt ? __crypto_aes_encrypt_hex : __crypto_aes_decrypt_hex;
    if (typeof fn === 'undefined') throw new Error('Native AES bridge is unavailable');
    return __hexToBytes(fn(mode, __bytesToHex(keyBytes), __bytesToHex(ivBytes), __bytesToHex(dataBytes)));
}

function __evpKdf(passwordBytes, saltBytes, keySizeBytes, ivSizeBytes) {
    var targetSize = keySizeBytes + ivSizeBytes;
    var derived = new Uint8Array(targetSize);
    var block = new Uint8Array(0);
    var offset = 0;
    while (offset < targetSize) {
        block = __nativeDigestBytes('MD5', __concatBytes(block, passwordBytes, saltBytes || new Uint8Array(0)));
        var take = Math.min(block.length, targetSize - offset);
        derived.set(block.subarray(0, take), offset);
        offset += take;
    }
    return {
        key: derived.subarray(0, keySizeBytes),
        iv: derived.subarray(keySizeBytes, keySizeBytes + ivSizeBytes)
    };
}

function __opensslSaltHeader() {
    return new Uint8Array([83, 97, 108, 116, 101, 100, 95, 95]);
}

function __hasOpenSslSaltHeader(bytes) {
    var header = __opensslSaltHeader();
    if (!bytes || bytes.length < 16) return false;
    for (var i = 0; i < header.length; i++) {
        if (bytes[i] !== header[i]) return false;
    }
    return true;
}

function __makeCipherParams(ciphertext, key, iv, salt, mode) {
    return {
        ciphertext: __bytesToWordArray(ciphertext),
        key: key ? __bytesToWordArray(key) : undefined,
        iv: iv ? __bytesToWordArray(iv) : undefined,
        salt: salt ? __bytesToWordArray(salt) : undefined,
        mode: mode,
        toString: function(formatter) {
            return (formatter || CryptoJS.format.OpenSSL).stringify(this);
        }
    };
}

var CryptoJS = {
    enc: {
        Hex: {
            stringify: function(wordArray) {
                return __bytesToHex(__wordArrayToBytes(wordArray));
            },
            parse: function(hexStr) {
                return __bytesToWordArray(__hexToBytes(hexStr));
            }
        },
        Utf8: {
            stringify: function(wordArray) {
                return new TextDecoder('utf-8').decode(__wordArrayToBytes(wordArray));
            },
            parse: function(utf8Str) {
                return __bytesToWordArray(new TextEncoder().encode(String(utf8Str)));
            }
        },
        Latin1: {
            stringify: function(wordArray) {
                var bytes = __wordArrayToBytes(wordArray);
                var out = '';
                for (var i = 0; i < bytes.length; i++) out += String.fromCharCode(bytes[i]);
                return out;
            },
            parse: function(str) {
                str = String(str || '');
                var bytes = new Uint8Array(str.length);
                for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0xff;
                return __bytesToWordArray(bytes);
            }
        },
        Base64: {
            stringify: function(wordArray) {
                var bytes = __wordArrayToBytes(wordArray);
                var binaryStr = '';
                for (var j = 0; j < bytes.length; j++) binaryStr += String.fromCharCode(bytes[j]);
                return btoa(binaryStr);
            },
            parse: function(base64Str) {
                var binaryStr = atob(String(base64Str || ''));
                var bytes = new Uint8Array(binaryStr.length);
                for (var i = 0; i < binaryStr.length; i++) bytes[i] = binaryStr.charCodeAt(i) & 0xff;
                return __bytesToWordArray(bytes);
            }
        },
        Base64url: {
            stringify: function(wordArray) {
                return CryptoJS.enc.Base64.stringify(wordArray).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
            },
            parse: function(str) {
                str = String(str || '').replace(/-/g, '+').replace(/_/g, '/');
                while (str.length % 4) str += '=';
                return CryptoJS.enc.Base64.parse(str);
            }
        }
    },
    lib: {
        WordArray: {
            create: function(words, sigBytes) {
                if (words == null) return __wordArrayCreate([], sigBytes || 0);
                if (__isWordArray(words)) return words.clone();
                if (typeof words === 'string') return CryptoJS.enc.Utf8.parse(words);
                if (words instanceof ArrayBuffer || (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView && ArrayBuffer.isView(words))) {
                    var bytes = __toUint8Array(words);
                    return __bytesToWordArray(sigBytes != undefined ? bytes.subarray(0, sigBytes) : bytes);
                }
                return __wordArrayCreate(words, sigBytes);
            },
            random: function(nBytes) {
                var bytes = new Uint8Array(nBytes || 0);
                globalThis.crypto.getRandomValues(bytes);
                return __bytesToWordArray(bytes);
            }
        },
        CipherParams: {
            create: function(params) {
                params = params || {};
                params.toString = params.toString || function(formatter) {
                    return (formatter || CryptoJS.format.OpenSSL).stringify(this);
                };
                return params;
            }
        }
    },
    format: {
        OpenSSL: {
            stringify: function(cipherParams) {
                var cipherBytes = __wordArrayToBytes(cipherParams.ciphertext);
                var out = cipherParams.salt
                    ? __concatBytes(__opensslSaltHeader(), __wordArrayToBytes(cipherParams.salt), cipherBytes)
                    : cipherBytes;
                return CryptoJS.enc.Base64.stringify(__bytesToWordArray(out));
            },
            parse: function(str) {
                var bytes = __wordArrayToBytes(CryptoJS.enc.Base64.parse(str));
                if (__hasOpenSslSaltHeader(bytes)) {
                    return CryptoJS.lib.CipherParams.create({
                        salt: __bytesToWordArray(bytes.subarray(8, 16)),
                        ciphertext: __bytesToWordArray(bytes.subarray(16))
                    });
                }
                return CryptoJS.lib.CipherParams.create({ ciphertext: __bytesToWordArray(bytes) });
            }
        }
    },
    mode: { CBC: 'AES-CBC', GCM: 'AES-GCM', ECB: 'AES-ECB' },
    pad: { Pkcs7: 'Pkcs7', NoPadding: 'NoPadding' },
    algo: { MD5: 'MD5', SHA1: 'SHA1', SHA256: 'SHA256', SHA384: 'SHA384', SHA512: 'SHA512', AES: 'AES' },
    MD5: function(m) { return __bytesToWordArray(__nativeDigestBytes('MD5', __normalizeWordArrayInput(m))); },
    SHA1: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA1', __normalizeWordArrayInput(m))); },
    SHA256: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA256', __normalizeWordArrayInput(m))); },
    SHA384: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA384', __normalizeWordArrayInput(m))); },
    SHA512: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA512', __normalizeWordArrayInput(m))); },
    HmacMD5: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('MD5', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA1: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA1', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA256: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA256', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA384: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA384', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA512: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA512', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    PBKDF2: function(pass, salt, options) {
        options = options || {};
        var pBytes = __normalizeWordArrayInput(pass);
        var sBytes = __normalizeWordArrayInput(salt);
        var iter = options.iterations || 1000;
        var kSize = options.keySize || 8;
        var algo = options.hasher || 'SHA1';
        return __bytesToWordArray(__nativePbkdf2Bytes(pBytes, sBytes, iter, kSize * 32, algo));
    },
    AES: {
        encrypt: function(message, key, options) {
            options = options || {};
            var data = __normalizeWordArrayInput(message);
            var kBytes;
            var ivBytes;
            var saltBytes;
            var isPassphrase = typeof key === 'string';
            if (isPassphrase) {
                saltBytes = options.salt ? __wordArrayToBytes(options.salt) : __wordArrayToBytes(CryptoJS.lib.WordArray.random(8));
                var derived = __evpKdf(new TextEncoder().encode(key), saltBytes, 32, 16);
                kBytes = derived.key;
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : derived.iv;
            } else {
                kBytes = __wordArrayToBytes(key);
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : new Uint8Array(0);
            }
            var mode = __aesModeName(options.mode || 'AES-CBC', options.padding);
            var resBytes = __nativeAesBytes(true, mode, kBytes, ivBytes, data);
            return __makeCipherParams(resBytes, kBytes, ivBytes, saltBytes, mode);
        },
        decrypt: function(cipher, key, options) {
            options = options || {};
            var cipherParams = typeof cipher === 'string' ? CryptoJS.format.OpenSSL.parse(cipher) : cipher;
            var data = cipherParams.ciphertext ? __wordArrayToBytes(cipherParams.ciphertext) : __toUint8Array(cipherParams);
            var kBytes;
            var ivBytes;
            var isPassphrase = typeof key === 'string';
            if (isPassphrase) {
                var saltBytes = options.salt ? __wordArrayToBytes(options.salt) : (cipherParams.salt ? __wordArrayToBytes(cipherParams.salt) : new Uint8Array(0));
                var derived = __evpKdf(new TextEncoder().encode(key), saltBytes, 32, 16);
                kBytes = derived.key;
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : derived.iv;
            } else {
                kBytes = __wordArrayToBytes(key);
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : new Uint8Array(0);
            }
            var mode = __aesModeName(options.mode || 'AES-CBC', options.padding);
            return __bytesToWordArray(__nativeAesBytes(false, mode, kBytes, ivBytes, data));
        }
    }
};
globalThis.CryptoJS = CryptoJS;

function __makeCryptoKey(type, algorithm, extractable, usages, rawBytes) {
    return {
        type: type,
        extractable: !!extractable,
        algorithm: algorithm,
        usages: usages || [],
        _raw: __copyUint8Array(rawBytes)
    };
}

function __webCryptoAlgorithm(algo) {
    var name = __normalizeAlgorithmName(algo);
    var out = { name: name };
    if (algo && typeof algo === 'object' && algo.length) out.length = algo.length;
    if (algo && typeof algo === 'object' && algo.hash) out.hash = { name: __normalizeHashName(algo.hash) };
    return out;
}

function __signatureAlgorithmName(algo, key) {
    var name = __normalizeAlgorithmName(algo || (key && key.algorithm));
    var hash = algo && algo.hash ? __normalizeHashName(algo.hash) : (key && key.algorithm && key.algorithm.hash ? key.algorithm.hash.name : 'SHA256');
    if (name === 'RSASSA-PKCS1-V1_5') return 'RSASSA-PKCS1-V1_5-' + hash;
    if (name === 'ECDSA') return 'ECDSA-' + hash;
    return name;
}

globalThis.crypto = {
    subtle: {
        digest: async function(algo, data) {
            return __bytesToArrayBuffer(__nativeDigestBytes(algo, __toUint8Array(data)));
        },
        importKey: async function(fmt, data, algo, extractable, usages) {
            fmt = String(fmt || 'raw').toLowerCase();
            if (fmt !== 'raw' && fmt !== 'pkcs8' && fmt !== 'spki') throw new Error('Unsupported key format: ' + fmt);
            var algorithm = __webCryptoAlgorithm(algo || {});
            var type = fmt === 'spki' ? 'public' : (fmt === 'pkcs8' ? 'private' : 'secret');
            return __makeCryptoKey(type, algorithm, extractable, usages || [], __toUint8Array(data));
        },
        exportKey: async function(fmt, key) {
            fmt = String(fmt || 'raw').toLowerCase();
            if (fmt !== 'raw' && fmt !== 'pkcs8' && fmt !== 'spki') throw new Error('Unsupported key format: ' + fmt);
            return __bytesToArrayBuffer(key._raw);
        },
        generateKey: async function(algo, extractable, usages) {
            var algorithm = __webCryptoAlgorithm(algo || {});
            if (algorithm.name !== 'AES-CBC' && algorithm.name !== 'AES-GCM' && algorithm.name !== 'HMAC') {
                throw new Error('Unsupported generateKey algorithm: ' + algorithm.name);
            }
            var length = algorithm.length || 256;
            var bytes = new Uint8Array(length / 8);
            globalThis.crypto.getRandomValues(bytes);
            return __makeCryptoKey('secret', algorithm, extractable, usages || [], bytes);
        },
        deriveBits: async function(params, key, len) {
            if (__normalizeAlgorithmName(params) !== 'PBKDF2') throw new Error('Only PBKDF2 deriveBits is supported');
            var pBytes = __toUint8Array(key._raw);
            var sBytes = __toUint8Array(params.salt);
            var hash = params.hash || 'SHA-256';
            return __bytesToArrayBuffer(__nativePbkdf2Bytes(pBytes, sBytes, params.iterations || 1000, len, hash));
        },
        deriveKey: async function(params, key, derivedKeyAlgo, extractable, usages) {
            var algorithm = __webCryptoAlgorithm(derivedKeyAlgo || {});
            var length = algorithm.length || 256;
            var raw = await globalThis.crypto.subtle.deriveBits(params, key, length);
            return __makeCryptoKey('secret', algorithm, extractable, usages || [], new Uint8Array(raw));
        },
        encrypt: async function(params, key, data) {
            var mode = __normalizeAlgorithmName(params);
            if (mode !== 'AES-CBC' && mode !== 'AES-GCM') throw new Error('Unsupported encrypt algorithm: ' + mode);
            var ivBytes = __toUint8Array(params.iv || new Uint8Array(0));
            return __bytesToArrayBuffer(__nativeAesBytes(true, mode, __toUint8Array(key._raw), ivBytes, __toUint8Array(data)));
        },
        decrypt: async function(params, key, data) {
            var mode = __normalizeAlgorithmName(params);
            if (mode !== 'AES-CBC' && mode !== 'AES-GCM') throw new Error('Unsupported decrypt algorithm: ' + mode);
            var ivBytes = __toUint8Array(params.iv || new Uint8Array(0));
            return __bytesToArrayBuffer(__nativeAesBytes(false, mode, __toUint8Array(key._raw), ivBytes, __toUint8Array(data)));
        },
        sign: async function(algo, key, data) {
            if (__normalizeAlgorithmName(algo || key.algorithm) === 'HMAC' || key.algorithm.name === 'HMAC') {
                var hash = (algo && algo.hash) || (key.algorithm && key.algorithm.hash) || 'SHA-256';
                return __bytesToArrayBuffer(__nativeHmacBytes(hash, __toUint8Array(key._raw), __toUint8Array(data)));
            }
            if (typeof __crypto_sign_hex === 'undefined') throw new Error('Native signature bridge is unavailable');
            var sigHex = __crypto_sign_hex(__signatureAlgorithmName(algo, key), __bytesToHex(key._raw), __bytesToHex(__toUint8Array(data)));
            return __bytesToArrayBuffer(__hexToBytes(sigHex));
        },
        verify: async function(algo, key, sig, data) {
            if (__normalizeAlgorithmName(algo || key.algorithm) === 'HMAC' || key.algorithm.name === 'HMAC') {
                var expected = __nativeHmacBytes((algo && algo.hash) || (key.algorithm && key.algorithm.hash) || 'SHA-256', __toUint8Array(key._raw), __toUint8Array(data));
                var actual = __toUint8Array(sig);
                if (expected.length !== actual.length) return false;
                var diff = 0;
                for (var i = 0; i < expected.length; i++) diff |= expected[i] ^ actual[i];
                return diff === 0;
            }
            if (typeof __crypto_verify_hex === 'undefined') throw new Error('Native signature bridge is unavailable');
            return __crypto_verify_hex(__signatureAlgorithmName(algo, key), __bytesToHex(key._raw), __bytesToHex(__toUint8Array(sig)), __bytesToHex(__toUint8Array(data)));
        }
    },
    getRandomValues: function(arr) {
        if (!arr) return arr;
        var byteLength = arr.byteLength != undefined ? arr.byteLength : arr.length;
        if (!byteLength) return arr;
        if (typeof __crypto_get_random_values_hex === 'undefined') throw new Error('Native random bridge is unavailable');
        var random = __hexToBytes(__crypto_get_random_values_hex(byteLength));
        if (arr.buffer && arr.byteLength != undefined) {
            new Uint8Array(arr.buffer, arr.byteOffset || 0, arr.byteLength).set(random);
        } else {
            for (var i = 0; i < arr.length; i++) arr[i] = random[i] || 0;
        }
        return arr;
    },
    randomUUID: function() {
        var b = new Uint8Array(16);
        globalThis.crypto.getRandomValues(b);
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        var h = __bytesToHex(b);
        return h.substr(0, 8) + '-' + h.substr(8, 4) + '-' + h.substr(12, 4) + '-' + h.substr(16, 4) + '-' + h.substr(20);
    }
};
"#;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scraper_can_use_cheerio_and_module_exports_without_network() {
        let code = r#"
            module.exports.getStreams = async function(tmdbId, mediaType) {
                var $ = cheerio.load('<div class="row" data-q="1080p">Alpha</div>');
                var el = $('.row');
                return [{ title: el.attr('data-q'), url: 'https://example.com/' + tmdbId + '-' + mediaType }];
            };
        "#
        .to_string();

        let result =
            execute_scraper(code, "123".to_string(), "movie".to_string(), None, None).unwrap();
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
        "#
        .to_string();

        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn fetch_bridge_rejects_private_hosts() {
        let code = r#"
            module.exports.getStreams = async function() {
                var res = await fetch('http://127.0.0.1:9/secret');
                var parsed = await res.json();
                return [{ title: 'x', url: 'https://example.com/blocked', ok: res.ok }];
            };
        "#
        .to_string();

        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["ok"], false);
    }

    #[test]
    fn cryptojs_sha256_matches_known_vector() {
        // sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let code = r#"
            module.exports.getStreams = async function() {
                var hash = CryptoJS.SHA256('abc').toString(CryptoJS.enc.Hex);
                return [{ title: hash, url: 'https://example.com/x' }];
            };
        "#
        .to_string();
        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed[0]["title"],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn cryptojs_hmac_sha256_matches_known_vector() {
        // HMAC-SHA256("key", "The quick brown fox jumps over the lazy dog")
        let code = r#"
            module.exports.getStreams = async function() {
                var mac = CryptoJS.HmacSHA256('The quick brown fox jumps over the lazy dog', 'key').toString(CryptoJS.enc.Hex);
                return [{ title: mac, url: 'https://example.com/x' }];
            };
        "#
        .to_string();
        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
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
        "#
        .to_string();
        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
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
        "#
        .to_string();
        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
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
        let result =
            execute_scraper(code, "1".to_string(), "movie".to_string(), None, None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["title"], "true");
    }
}
