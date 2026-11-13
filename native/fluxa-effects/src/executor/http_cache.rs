use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use web_time::Instant;

const CAPACITY: usize = 256;
const MAX_BODY: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Directives {
    max_age: u64,
    stale_revalidate: u64,
    stale_error: u64,
}

struct Entry {
    stored_at: Instant,
    directives: Directives,
    status: i32,
    body: String,
    refreshing: bool,
}

pub(super) enum Lookup {
    Fresh(i32, String),
    Stale(i32, String),
    Miss,
}

fn entries() -> &'static Mutex<HashMap<String, Entry>> {
    static ENTRIES: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
    ENTRIES.get_or_init(Default::default)
}

pub(super) fn parse(header: &str) -> Option<Directives> {
    let mut directives = Directives::default();
    for part in header.split(',') {
        let part = part.trim().to_ascii_lowercase();
        if part == "no-store" || part == "no-cache" || part == "private" {
            return None;
        }
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let seconds = value.trim().trim_matches('"').parse().unwrap_or(0);
        match name.trim() {
            "max-age" => directives.max_age = seconds,
            "stale-while-revalidate" => directives.stale_revalidate = seconds,
            "stale-if-error" => directives.stale_error = seconds,
            _ => {}
        }
    }
    (directives.max_age > 0).then_some(directives)
}

pub(super) fn lookup(url: &str) -> Lookup {
    let mut entries = entries().lock().unwrap_or_else(|error| error.into_inner());
    let Some(entry) = entries.get_mut(url) else {
        return Lookup::Miss;
    };
    let age = entry.stored_at.elapsed();
    let fresh = Duration::from_secs(entry.directives.max_age);
    if age < fresh {
        return Lookup::Fresh(entry.status, entry.body.clone());
    }
    let revalidate = fresh + Duration::from_secs(entry.directives.stale_revalidate);
    if age < revalidate && !entry.refreshing {
        entry.refreshing = true;
        return Lookup::Stale(entry.status, entry.body.clone());
    }
    Lookup::Miss
}

pub(super) fn stale_on_error(url: &str) -> Option<(i32, String)> {
    let entries = entries().lock().unwrap_or_else(|error| error.into_inner());
    let entry = entries.get(url)?;
    let window = Duration::from_secs(entry.directives.max_age + entry.directives.stale_error);
    (entry.stored_at.elapsed() < window).then(|| (entry.status, entry.body.clone()))
}

pub(super) fn store(url: &str, directives: Directives, status: i32, body: &str) {
    if status != 200 || body.len() > MAX_BODY {
        return;
    }
    let mut entries = entries().lock().unwrap_or_else(|error| error.into_inner());
    if entries.len() >= CAPACITY
        && !entries.contains_key(url)
        && let Some(oldest) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.stored_at)
            .map(|(key, _)| key.clone())
    {
        entries.remove(&oldest);
    }
    entries.insert(
        url.to_owned(),
        Entry {
            stored_at: Instant::now(),
            directives,
            status,
            body: body.to_owned(),
            refreshing: false,
        },
    );
}

pub(super) fn refresh_done(url: &str) {
    let mut entries = entries().lock().unwrap_or_else(|error| error.into_inner());
    if let Some(entry) = entries.get_mut(url) {
        entry.refreshing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_header_is_parsed() {
        let parsed = parse("max-age=300, stale-while-revalidate=1800, stale-if-error=604800");
        assert_eq!(
            parsed,
            Some(Directives {
                max_age: 300,
                stale_revalidate: 1800,
                stale_error: 604800
            })
        );
    }

    #[test]
    fn no_store_and_zero_max_age_are_not_cached() {
        assert_eq!(parse("no-store"), None);
        assert_eq!(parse("public, max-age=0"), None);
        assert_eq!(parse("stale-while-revalidate=60"), None);
    }

    #[test]
    fn fresh_entry_is_served_and_stale_error_window_applies() {
        let url = "https://cache-test.example.com/a.json";
        let directives = parse("max-age=60, stale-if-error=600").unwrap();
        store(url, directives, 200, "{}");
        assert!(matches!(lookup(url), Lookup::Fresh(200, _)));
        assert!(stale_on_error(url).is_some());
        assert!(matches!(
            lookup("https://cache-test.example.com/b.json"),
            Lookup::Miss
        ));
    }

    #[test]
    fn error_responses_are_never_stored() {
        let url = "https://cache-test.example.com/c.json";
        store(url, parse("max-age=60").unwrap(), 500, "oops");
        assert!(matches!(lookup(url), Lookup::Miss));
    }
}
