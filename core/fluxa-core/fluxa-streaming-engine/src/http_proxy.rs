use axum::body::Body;
use axum::extract::{Query, Request};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use reqwest::header::{HeaderMap, HeaderName};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

#[derive(Deserialize)]
pub struct ProxyQuery {
    url: String,
    h: Option<String>,
}

fn allowed_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => {
            let octets = value.octets();
            !value.is_loopback()
                && !value.is_private()
                && !value.is_link_local()
                && !value.is_unspecified()
                && !value.is_multicast()
                && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
        }
        IpAddr::V6(value) => !value.is_loopback()
            && !value.is_unspecified()
            && !value.is_multicast()
            && value.segments()[0] & 0xfe00 != 0xfc00,
    }
}

async fn resolve_public(url: &url::Url) -> Result<SocketAddr, Response> {
    let host = url.host_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "url has no host").into_response())?;
    let port = url.port_or_known_default().ok_or_else(|| (StatusCode::BAD_REQUEST, "url has no port").into_response())?;
    let mut addresses = tokio::net::lookup_host((host, port)).await.map_err(|_| (StatusCode::BAD_REQUEST, "host cannot be resolved").into_response())?;
    addresses.find(|address| allowed_ip(address.ip())).ok_or_else(|| (StatusCode::BAD_REQUEST, "private hosts are not allowed").into_response())
}

fn requested_headers(raw: Option<&str>) -> HeaderMap {
    let values = raw.and_then(|value| serde_json::from_str::<HashMap<String, String>>(value).ok()).unwrap_or_default();
    let mut headers = HeaderMap::new();
    for (name, value) in values {
        let Ok(name) = HeaderName::try_from(name) else { continue };
        let Ok(value) = HeaderValue::try_from(value) else { continue };
        if name == header::REFERER || name == header::USER_AGENT { headers.insert(name, value); }
    }
    headers
}

pub async fn handle_proxy(Query(query): Query<ProxyQuery>, request: Request) -> Response {
    let url = match url::Url::parse(&query.url) {
        Ok(url) if url.scheme() == "http" || url.scheme() == "https" => url,
        _ => return (StatusCode::BAD_REQUEST, "url must use http or https").into_response(),
    };
    let address = match resolve_public(&url).await { Ok(address) => address, Err(response) => return response };
    let host = match url.host_str() { Some(host) => host, None => return (StatusCode::BAD_REQUEST, "url has no host").into_response() };
    let client = match reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).resolve(host, address).build() {
        Ok(client) => client,
        Err(error) => return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    };
    let mut headers = requested_headers(query.h.as_deref());
    if let Some(range) = request.headers().get(header::RANGE) { headers.insert(header::RANGE, range.clone()); }
    let response = match client.get(url).headers(headers).send().await {
        Ok(response) => response,
        Err(error) => return (StatusCode::BAD_GATEWAY, error.to_string()).into_response(),
    };
    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let content_type = response.headers().get(header::CONTENT_TYPE).cloned();
    if content_type.as_ref().and_then(|value| value.to_str().ok()).is_some_and(|value| value.starts_with("text/html")) {
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "upstream is not media").into_response();
    }
    let content_length = response.headers().get(header::CONTENT_LENGTH).cloned();
    let body = Body::from_stream(response.bytes_stream());
    let mut output = (status, body).into_response();
    if let Some(value) = content_type { output.headers_mut().insert(header::CONTENT_TYPE, value); }
    if let Some(value) = content_length { output.headers_mut().insert(header::CONTENT_LENGTH, value); }
    output.headers_mut().insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    output
}

pub fn router() -> Router { Router::new().route("/proxy", get(handle_proxy)) }
