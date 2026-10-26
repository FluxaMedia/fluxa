use fluxa_core::addons::plugin_challenge::{
    CLEARANCE_TTL_SECS, has_clearance, merge_cookie_header,
};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SOLVE_TIMEOUT: Duration = Duration::from_secs(18);
const STORE_KEY: &str = "cf_clearance";
const CLEAN_SECS: u64 = 10 * 60;
const MIN_LIFETIME_SECS: u64 = 60;

#[derive(Clone)]
pub(crate) struct Clearance {
    cookies: Vec<(String, String)>,
    user_agent: String,
    expires_at: u64,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

struct Broker {
    available: AtomicBool,
    next_id: AtomicU64,
    queue: Mutex<VecDeque<(u64, String)>>,
    wake: Condvar,
    waiting: Mutex<HashMap<u64, Sender<Option<Clearance>>>>,
    cache: Mutex<HashMap<String, Clearance>>,
    clean: Mutex<HashMap<String, Instant>>,
    solving: Mutex<()>,
}

fn broker() -> &'static Broker {
    static BROKER: OnceLock<Broker> = OnceLock::new();
    BROKER.get_or_init(|| Broker {
        available: AtomicBool::new(false),
        next_id: AtomicU64::new(1),
        queue: Mutex::default(),
        wake: Condvar::new(),
        waiting: Mutex::default(),
        cache: Mutex::new(load_saved()),
        clean: Mutex::default(),
        solving: Mutex::new(()),
    })
}

pub fn set_challenge_solver_available(available: bool) {
    broker().available.store(available, Ordering::Relaxed);
}

fn load_saved() -> HashMap<String, Clearance> {
    let saved = crate::storage::Storage::open_default()
        .ok()
        .and_then(|storage| storage.read_json(STORE_KEY).ok().flatten());
    let Some(serde_json::Value::Object(hosts)) = saved else {
        return HashMap::new();
    };
    hosts
        .into_iter()
        .filter_map(|(host, entry)| {
            let expires_at = entry.get("expiresAt")?.as_u64()?;
            let clearance = Clearance {
                cookies: parse_cookies(entry.get("cookies")?.as_str()?),
                user_agent: entry.get("userAgent")?.as_str()?.to_owned(),
                expires_at,
            };
            (expires_at > now() && has_clearance(&clearance.cookies)).then_some((host, clearance))
        })
        .collect()
}

fn save(cache: &HashMap<String, Clearance>) {
    let hosts = cache
        .iter()
        .filter(|(_, clearance)| clearance.expires_at > now())
        .map(|(host, clearance)| {
            let cookies = clearance
                .cookies
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; ");
            let entry = serde_json::json!({
                "cookies": cookies,
                "userAgent": clearance.user_agent,
                "expiresAt": clearance.expires_at,
            });
            (host.clone(), entry)
        })
        .collect::<serde_json::Map<_, _>>();
    if let Ok(storage) = crate::storage::Storage::open_default() {
        let _ = storage.write_json(STORE_KEY, &serde_json::Value::Object(hosts));
    }
}

pub fn next_challenge_request(timeout_ms: u64) -> Option<(u64, String)> {
    let broker = broker();
    let mut queue = broker.queue.lock().ok()?;
    if queue.is_empty() {
        queue = broker
            .wake
            .wait_timeout(queue, Duration::from_millis(timeout_ms))
            .ok()?
            .0;
    }
    queue.pop_front()
}

pub fn complete_challenge(id: u64, cookie_header: &str, user_agent: &str, expires_at: u64) {
    let expires_at = if expires_at > now() + MIN_LIFETIME_SECS {
        expires_at
    } else {
        now() + CLEARANCE_TTL_SECS
    };
    let clearance = Some(Clearance {
        cookies: parse_cookies(cookie_header),
        user_agent: user_agent.to_owned(),
        expires_at,
    })
    .filter(|clearance| has_clearance(&clearance.cookies) && !clearance.user_agent.is_empty());
    let sender = broker()
        .waiting
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&id));
    if let Some(sender) = sender {
        let _ = sender.send(clearance);
    }
}

fn parse_cookies(header: &str) -> Vec<(String, String)> {
    header
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

pub(crate) fn cached(host: &str) -> Option<Clearance> {
    let mut cache = broker().cache.lock().ok()?;
    match cache.get(host) {
        Some(clearance) if clearance.expires_at > now() => Some(clearance.clone()),
        Some(_) => {
            cache.remove(host);
            save(&cache);
            None
        }
        None => None,
    }
}

pub(crate) fn forget(host: &str) {
    if let Ok(mut cache) = broker().cache.lock()
        && cache.remove(host).is_some()
    {
        save(&cache);
    }
}

pub(crate) fn recently_clean(host: &str) -> bool {
    broker().clean.lock().is_ok_and(|clean| {
        clean
            .get(host)
            .is_some_and(|at| at.elapsed().as_secs() < CLEAN_SECS)
    })
}

pub(crate) fn mark_clean(host: &str) {
    if let Ok(mut clean) = broker().clean.lock() {
        clean.insert(host.to_owned(), Instant::now());
    }
}

pub(crate) fn merged_headers(
    base: &[(String, String)],
    clearance: &Clearance,
) -> Vec<(String, String)> {
    let cookie = base
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("cookie"))
        .map(|(_, value)| value.as_str());
    let mut merged: Vec<(String, String)> = base
        .iter()
        .filter(|(name, _)| {
            !name.eq_ignore_ascii_case("cookie") && !name.eq_ignore_ascii_case("user-agent")
        })
        .cloned()
        .collect();
    merged.push((
        "Cookie".to_owned(),
        merge_cookie_header(cookie, &clearance.cookies),
    ));
    merged.push(("User-Agent".to_owned(), clearance.user_agent.clone()));
    merged
}

pub(crate) fn solver_available() -> bool {
    broker().available.load(Ordering::Relaxed)
}

pub fn set_stream_headers(url: &str, headers: Vec<(String, String)>) {
    fluxa_mpv::set_request_headers(crate::executor::stream_request_headers);
    if let Ok(mut stored) = stream_headers_slot().lock() {
        *stored = Some((url.to_owned(), headers));
    }
}

pub(crate) fn stored_stream_headers(url: &str) -> Vec<(String, String)> {
    stream_headers_slot()
        .lock()
        .ok()
        .and_then(|stored| stored.clone())
        .filter(|(stored_url, _)| stored_url == url)
        .map(|(_, headers)| headers)
        .unwrap_or_default()
}

fn stream_headers_slot() -> &'static Mutex<Option<(String, Vec<(String, String)>)>> {
    static SLOT: OnceLock<Mutex<Option<(String, Vec<(String, String)>)>>> = OnceLock::new();
    SLOT.get_or_init(Default::default)
}

pub(crate) fn solve(url: &str, host: &str) -> Option<Clearance> {
    let broker = broker();
    if !broker.available.load(Ordering::Relaxed) {
        return None;
    }
    let _turn = broker.solving.lock().ok()?;
    if let Some(fresh) = cached(host) {
        return Some(fresh);
    }
    let id = broker.next_id.fetch_add(1, Ordering::Relaxed);
    let (sender, receiver) = channel();
    broker.waiting.lock().ok()?.insert(id, sender);
    broker.queue.lock().ok()?.push_back((id, url.to_owned()));
    broker.wake.notify_one();
    let clearance = receiver.recv_timeout(SOLVE_TIMEOUT).ok().flatten();
    if let Ok(mut waiting) = broker.waiting.lock() {
        waiting.remove(&id);
    }
    if let Ok(mut queue) = broker.queue.lock() {
        queue.retain(|(queued, _)| *queued != id);
    }
    let clearance = clearance?;
    if let Ok(mut cache) = broker.cache.lock() {
        cache.insert(host.to_owned(), clearance.clone());
        save(&cache);
    }
    Some(clearance)
}

pub(crate) fn apply(headers: &mut HashMap<String, String>, clearance: &Clearance) {
    let existing = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("cookie"))
        .map(|(_, value)| value.clone());
    headers.retain(|name, _| {
        !name.eq_ignore_ascii_case("cookie") && !name.eq_ignore_ascii_case("user-agent")
    });
    headers.insert(
        "cookie".to_owned(),
        merge_cookie_header(existing.as_deref(), &clearance.cookies),
    );
    headers.insert("user-agent".to_owned(), clearance.user_agent.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearance_without_cf_cookie_is_rejected() {
        let id = 9001;
        let (sender, receiver) = channel();
        broker().waiting.lock().unwrap().insert(id, sender);
        complete_challenge(id, "a=1", "Mozilla/5.0", 0);
        assert!(receiver.recv().unwrap().is_none());
    }

    #[test]
    fn solved_clearance_overrides_cookie_and_agent() {
        let mut headers = HashMap::from([
            ("Cookie".to_owned(), "a=1".to_owned()),
            ("User-Agent".to_owned(), "plugin".to_owned()),
        ]);
        let clearance = Clearance {
            cookies: vec![("cf_clearance".to_owned(), "tok".to_owned())],
            user_agent: "Mozilla/5.0".to_owned(),
            expires_at: 0,
        };
        apply(&mut headers, &clearance);
        assert_eq!(headers["cookie"], "a=1; cf_clearance=tok");
        assert_eq!(headers["user-agent"], "Mozilla/5.0");
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn stream_headers_keep_referer_and_take_clearance_cookie() {
        let clearance = Clearance {
            cookies: vec![("cf_clearance".to_owned(), "tok".to_owned())],
            user_agent: "Mozilla/5.0".to_owned(),
            expires_at: 0,
        };
        let base = vec![
            ("Referer".to_owned(), "https://site/".to_owned()),
            ("user-agent".to_owned(), "plugin".to_owned()),
            ("Cookie".to_owned(), "a=1".to_owned()),
        ];
        let merged = merged_headers(&base, &clearance);
        assert!(merged.contains(&("Referer".to_owned(), "https://site/".to_owned())));
        assert!(merged.contains(&("Cookie".to_owned(), "a=1; cf_clearance=tok".to_owned())));
        assert_eq!(
            merged
                .iter()
                .filter(|(name, _)| name.eq_ignore_ascii_case("user-agent"))
                .count(),
            1
        );
    }

    #[test]
    fn request_without_pump_is_not_queued() {
        set_challenge_solver_available(false);
        assert!(solve("https://example.com/", "example.com").is_none());
        assert!(next_challenge_request(0).is_none());
    }
}
