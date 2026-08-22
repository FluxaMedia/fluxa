use super::dom_bridge::DomBridge;
use super::{
    PLUGIN_MAX_RESPONSE_BODY_BYTES, PluginHttpClient, PluginHttpRequest, crypto_bridge,
    plugin_http_request_error,
};
use rquickjs::{Ctx, Function};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

fn native_fetch(
    client: &Arc<dyn PluginHttpClient>,
    url: String,
    method: String,
    headers_json: String,
    body: Option<String>,
    follow_redirects: bool,
) -> String {
    let headers: HashMap<String, String> = match serde_json::from_str(&headers_json) {
        Ok(headers) => headers,
        Err(_) => {
            return "{\"ok\":false,\"status\":0,\"body\":\"\",\"error\":\"invalid plugin request headers\"}".to_string();
        }
    };
    let request = PluginHttpRequest {
        method,
        url,
        headers,
        body,
        follow_redirects,
    };
    if let Some(error) = plugin_http_request_error(&request) {
        return format!(
            "{{\"ok\":false,\"status\":0,\"body\":\"\",\"error\":{}}}",
            serde_json::to_string(error).unwrap_or_else(|_| "\"plugin request rejected\"".into())
        );
    }
    let mut response = client.fetch(request);
    if response.body.len() > PLUGIN_MAX_RESPONSE_BODY_BYTES {
        response.body.truncate(PLUGIN_MAX_RESPONSE_BODY_BYTES);
        response.ok = false;
        response.error = Some("plugin response body exceeds limit".to_string());
    }
    format!(
        "{{\"ok\":{},\"status\":{},\"body\":{},\"headers\":{},\"error\":{}}}",
        response.ok,
        response.status,
        serde_json::to_string(&response.body).unwrap_or_else(|_| "\"\"".into()),
        serde_json::to_string(&response.headers).unwrap_or_else(|_| "{}".into()),
        serde_json::to_string(&response.error).unwrap_or_else(|_| "null".into())
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

pub(super) fn register_host_functions(
    ctx: &Ctx<'_>,
    dom: &Rc<DomBridge>,
    client: Arc<dyn PluginHttpClient>,
) -> rquickjs::Result<()> {
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
        Function::new(
            ctx.clone(),
            move |url: String,
                  method: String,
                  headers_json: String,
                  body: Option<String>,
                  follow_redirects: bool| {
                native_fetch(&client, url, method, headers_json, body, follow_redirects)
            },
        )?,
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

pub(super) const BASE64_POLYFILL: &str = include_str!("polyfills/base64.js");
pub(super) const TEXT_ENCODER_POLYFILL: &str = include_str!("polyfills/text_encoder.js");
pub(super) const CRYPTO_POLYFILL: &str = include_str!("polyfills/crypto.js");
pub(super) const CHEERIO_POLYFILL: &str = include_str!("polyfills/cheerio.js");
