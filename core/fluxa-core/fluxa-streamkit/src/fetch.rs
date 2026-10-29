use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

use crate::parse::{ParseEnv, ParseError, ParseOutcome, parser_for};
use crate::types::{AddonRef, ParsedStream, RawStream, StreamType};

#[derive(Clone, Debug, PartialEq)]
pub enum TransportError {
    Timeout,
    Network(String),
}

#[derive(Clone, Debug)]
pub struct TransportResponse {
    pub status: u16,
    pub body: String,
}

pub trait Transport {
    fn get(
        &self,
        url: &str,
        timeout: Duration,
    ) -> impl Future<Output = Result<TransportResponse, TransportError>>;
}

#[derive(Clone, Debug)]
pub struct AddonEndpoint {
    pub addon: AddonRef,
    pub manifest_url: String,
    pub timeout: Duration,
    pub media_types: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Merged,
    Error,
    CutOff,
    NotStarted,
}

#[derive(Clone, Debug)]
pub struct AddonReport {
    pub addon: String,
    pub outcome: Outcome,
    pub raw_count: usize,
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct AddonResult {
    pub report: AddonReport,
    pub streams: Vec<ParsedStream>,
}

pub fn stream_url(manifest_url: &str, media_type: &str, id: &str) -> String {
    let url = manifest_url.strip_prefix("stremio://").map_or_else(
        || manifest_url.to_string(),
        |rest| format!("https://{rest}"),
    );
    let without_query = url.split(['?', '#']).next().unwrap_or(&url);
    let base = without_query
        .rsplit_once('/')
        .map_or(without_query, |(b, _)| b);
    format!("{base}/stream/{media_type}/{}.json", encode_component(id))
}

fn encode_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

pub fn hostname(url: &str) -> Option<&str> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    let host = host.split(':').next()?;
    (!host.is_empty()).then_some(host)
}

pub fn parse_response(body: &str) -> Result<Vec<RawStream>, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("invalid JSON: {e}"))?;
    let items = value
        .get("streams")
        .and_then(Value::as_array)
        .ok_or_else(|| "response has no streams array".to_string())?;
    let valid: Vec<RawStream> = items
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    if !items.is_empty() && valid.is_empty() {
        return Err("no valid streams in response".to_string());
    }
    Ok(valid)
}

pub fn error_stream(addon: &AddonRef, message: &str) -> ParsedStream {
    let mut s = ParsedStream::blank(
        format!("{}:error", addon.instance_id),
        addon.clone(),
        StreamType::Error,
    );
    s.message = Some(format!("[❌] {}: {message}", addon.name));
    s
}

pub async fn fetch_addon<T: Transport>(
    transport: &T,
    endpoint: &AddonEndpoint,
    media_type: &str,
    id: &str,
    now_ms: i64,
) -> AddonResult {
    let addon = &endpoint.addon;
    let report = |outcome, raw_count, latency_ms, error: Option<String>| AddonReport {
        addon: addon.name.clone(),
        outcome,
        raw_count,
        latency_ms,
        error,
    };
    if !endpoint.media_types.is_empty() && !endpoint.media_types.iter().any(|t| t == media_type) {
        return AddonResult {
            report: report(Outcome::NotStarted, 0, 0, None),
            streams: Vec::new(),
        };
    }
    let url = stream_url(&endpoint.manifest_url, media_type, id);
    let started = web_time::Instant::now();
    let latency = || u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let failed = |message: String, outcome: Outcome| AddonResult {
        streams: vec![error_stream(addon, &message)],
        report: report(outcome, 0, latency(), Some(message)),
    };
    let response = match transport.get(&url, endpoint.timeout).await {
        Ok(r) => r,
        Err(TransportError::Timeout) => return failed("request timed out".into(), Outcome::Error),
        Err(TransportError::Network(e)) => return failed(e, Outcome::Error),
    };
    if !(200..300).contains(&response.status) {
        return failed(format!("HTTP {}", response.status), Outcome::Error);
    }
    let raws = match parse_response(&response.body) {
        Ok(r) => r,
        Err(e) => return failed(e, Outcome::Error),
    };
    let raw_count = raws.len();
    let parser = parser_for(&addon.preset_id);
    let env = ParseEnv { now_ms };
    let mut seq = 0;
    let mut streams = Vec::with_capacity(raw_count);
    for raw in raws {
        match parser.parse(raw, addon, &mut seq, &env) {
            Ok(ParseOutcome::Stream(s)) => streams.push(*s),
            Ok(ParseOutcome::Skip) | Err(ParseError::NotPlayable) => {}
            Err(ParseError::Addon(message)) => {
                let mut s = error_stream(addon, &message);
                s.id = format!("{}:error:{seq}", addon.instance_id);
                streams.push(s);
            }
        }
    }
    AddonResult {
        report: report(Outcome::Merged, raw_count, latency(), None),
        streams,
    }
}

pub fn cut_off(endpoint: &AddonEndpoint, started: bool) -> AddonReport {
    AddonReport {
        addon: endpoint.addon.name.clone(),
        outcome: if started {
            Outcome::CutOff
        } else {
            Outcome::NotStarted
        },
        raw_count: 0,
        latency_ms: 0,
        error: None,
    }
}

#[derive(Clone, Debug, Default)]
pub struct FailureCache {
    until: HashMap<String, i64>,
    pub hold_ms: i64,
}

impl FailureCache {
    pub fn new(hold_ms: i64) -> Self {
        Self {
            until: HashMap::new(),
            hold_ms,
        }
    }

    pub fn record(&mut self, manifest_url: &str, now_ms: i64) {
        self.until
            .insert(manifest_url.to_string(), now_ms + self.hold_ms);
    }

    pub fn is_blocked(&self, manifest_url: &str, now_ms: i64) -> bool {
        self.until
            .get(manifest_url)
            .is_some_and(|until| *until > now_ms)
    }
}

pub fn resolve_ttl(
    table: &HashMap<String, u64>,
    preset_id: &str,
    manifest_url: &str,
) -> Option<Duration> {
    let seconds = table
        .get(preset_id)
        .or_else(|| hostname(manifest_url).and_then(|h| table.get(h)))
        .or_else(|| table.get("*"))?;
    Some(Duration::from_secs(*seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::task::{Context, Poll, Waker};

    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = Box::pin(fut);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    struct Canned {
        reply: Result<TransportResponse, TransportError>,
        asked: RefCell<Vec<String>>,
    }

    impl Transport for Canned {
        async fn get(
            &self,
            url: &str,
            _timeout: Duration,
        ) -> Result<TransportResponse, TransportError> {
            self.asked.borrow_mut().push(url.to_string());
            self.reply.clone()
        }
    }

    fn endpoint(media_types: &[&str]) -> AddonEndpoint {
        AddonEndpoint {
            addon: AddonRef {
                instance_id: "i1".into(),
                preset_id: "custom".into(),
                name: "Alpha".into(),
                ..Default::default()
            },
            manifest_url: "stremio://alpha.example.com/token/manifest.json".into(),
            timeout: Duration::from_secs(5),
            media_types: media_types.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn canned(status: u16, body: &str) -> Canned {
        Canned {
            reply: Ok(TransportResponse {
                status,
                body: body.into(),
            }),
            asked: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn stream_url_swaps_scheme_and_encodes_the_id() {
        assert_eq!(
            stream_url(
                "stremio://alpha.example.com/token/manifest.json",
                "series",
                "tt1:2:3"
            ),
            "https://alpha.example.com/token/stream/series/tt1%3A2%3A3.json"
        );
    }

    #[test]
    fn invalid_items_are_dropped_one_by_one() {
        let body = r#"{"streams":[{"url":"https://x/a.mkv","title":"A.2020.1080p"},"junk",{"url":"https://x/b.mkv"}]}"#;
        assert_eq!(parse_response(body).unwrap().len(), 2);
    }

    #[test]
    fn all_invalid_items_fail_the_addon_but_empty_is_fine() {
        assert!(parse_response(r#"{"streams":["a","b"]}"#).is_err());
        assert_eq!(parse_response(r#"{"streams":[]}"#).unwrap().len(), 0);
        assert!(parse_response(r#"{"nope":1}"#).is_err());
    }

    #[test]
    fn successful_fetch_parses_streams_and_reports_counts() {
        let t = canned(
            200,
            r#"{"streams":[{"url":"https://x/a.mkv","title":"Movie.2020.1080p.WEB-DL-GRP\n👤 5"}]}"#,
        );
        let r = block_on(fetch_addon(&t, &endpoint(&[]), "movie", "tt1", 0));
        assert_eq!(r.report.outcome, Outcome::Merged);
        assert_eq!(r.report.raw_count, 1);
        assert_eq!(r.streams.len(), 1);
        assert_eq!(
            t.asked.borrow()[0],
            "https://alpha.example.com/token/stream/movie/tt1.json"
        );
    }

    #[test]
    fn http_errors_become_an_error_entry() {
        let r = block_on(fetch_addon(
            &canned(503, ""),
            &endpoint(&[]),
            "movie",
            "tt1",
            0,
        ));
        assert_eq!(r.report.outcome, Outcome::Error);
        assert_eq!(r.streams[0].stream_type, StreamType::Error);
        assert_eq!(
            r.streams[0].message.as_deref(),
            Some("[❌] Alpha: HTTP 503")
        );
    }

    #[test]
    fn timeout_is_an_error_entry_not_a_panic() {
        let t = Canned {
            reply: Err(TransportError::Timeout),
            asked: RefCell::new(Vec::new()),
        };
        let r = block_on(fetch_addon(&t, &endpoint(&[]), "movie", "tt1", 0));
        assert_eq!(r.report.error.as_deref(), Some("request timed out"));
    }

    #[test]
    fn media_type_gate_skips_without_a_request() {
        let t = canned(200, "{}");
        let r = block_on(fetch_addon(&t, &endpoint(&["series"]), "movie", "tt1", 0));
        assert_eq!(r.report.outcome, Outcome::NotStarted);
        assert!(t.asked.borrow().is_empty());
    }

    #[test]
    fn failure_cache_blocks_until_the_hold_expires() {
        let mut c = FailureCache::new(60_000);
        c.record("https://a/manifest.json", 1_000);
        assert!(c.is_blocked("https://a/manifest.json", 30_000));
        assert!(!c.is_blocked("https://a/manifest.json", 61_001));
    }

    #[test]
    fn ttl_prefers_preset_then_host_then_wildcard() {
        let table: HashMap<String, u64> = [
            ("torrentio".into(), 10),
            ("alpha.example.com".into(), 20),
            ("*".into(), 30),
        ]
        .into_iter()
        .collect();
        let url = "https://alpha.example.com/x/manifest.json";
        assert_eq!(
            resolve_ttl(&table, "torrentio", url),
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            resolve_ttl(&table, "other", url),
            Some(Duration::from_secs(20))
        );
        assert_eq!(
            resolve_ttl(&table, "other", "https://b.example.com/m.json"),
            Some(Duration::from_secs(30))
        );
        assert_eq!(resolve_ttl(&HashMap::new(), "other", url), None);
    }
}
