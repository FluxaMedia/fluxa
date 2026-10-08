use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn failing() -> &'static Mutex<HashMap<String, String>> {
    static FAILING: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    FAILING.get_or_init(Default::default)
}

fn authority(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or_default()
}

pub(crate) fn record(url: &str, failure: Option<String>) {
    let host = authority(url);
    if host.is_empty() {
        return;
    }
    let Ok(mut failing) = failing().lock() else {
        return;
    };
    match failure {
        Some(detail) => {
            failing.insert(host.to_owned(), detail);
        }
        None => {
            failing.remove(host);
        }
    }
}

pub fn unreachable_addon_hosts() -> Vec<(String, String)> {
    failing()
        .lock()
        .map(|failing| {
            failing
                .iter()
                .map(|(host, detail)| (host.clone(), detail.clone()))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn request_error(error: &reqwest::Error) -> String {
    let mut causes = Vec::new();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        causes.push(cause.to_string());
        source = cause.source();
    }
    if causes.is_empty() {
        error.to_string()
    } else {
        causes.join(": ")
    }
}
