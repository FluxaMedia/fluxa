use fluxa_core::addons::plugin_challenge::{
    CLEARANCE_TTL_SECS, has_clearance, merge_cookie_header,
};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

const SOLVE_TIMEOUT: Duration = Duration::from_secs(18);

#[derive(Clone)]
pub(crate) struct Clearance {
    cookies: Vec<(String, String)>,
    user_agent: String,
}

struct Broker {
    available: AtomicBool,
    next_id: AtomicU64,
    queue: Mutex<VecDeque<(u64, String)>>,
    wake: Condvar,
    waiting: Mutex<HashMap<u64, Sender<Option<Clearance>>>>,
    cache: Mutex<HashMap<String, (Clearance, Instant)>>,
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
        cache: Mutex::default(),
        solving: Mutex::new(()),
    })
}

pub fn set_challenge_solver_available(available: bool) {
    broker().available.store(available, Ordering::Relaxed);
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

pub fn complete_challenge(id: u64, cookie_header: &str, user_agent: &str) {
    let clearance = Some(Clearance {
        cookies: parse_cookies(cookie_header),
        user_agent: user_agent.to_owned(),
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
        Some((clearance, at)) if at.elapsed().as_secs() < CLEARANCE_TTL_SECS => {
            Some(clearance.clone())
        }
        Some(_) => {
            cache.remove(host);
            None
        }
        None => None,
    }
}

pub(crate) fn forget(host: &str) {
    if let Ok(mut cache) = broker().cache.lock() {
        cache.remove(host);
    }
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
        cache.insert(host.to_owned(), (clearance.clone(), Instant::now()));
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
        complete_challenge(id, "a=1", "Mozilla/5.0");
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
        };
        apply(&mut headers, &clearance);
        assert_eq!(headers["cookie"], "a=1; cf_clearance=tok");
        assert_eq!(headers["user-agent"], "Mozilla/5.0");
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn request_without_pump_is_not_queued() {
        set_challenge_solver_available(false);
        assert!(solve("https://example.com/", "example.com").is_none());
        assert!(next_challenge_request(0).is_none());
    }
}
