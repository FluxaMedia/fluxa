use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::luna::{self, Subscription};

const REGISTER: &str = "luna://com.webos.service.applicationmanager/registerApp";

const MINIMIZED_GRACE: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct Grace {
    since: Option<Instant>,
}

impl Grace {
    pub fn minimized(&mut self, now: Instant) {
        self.since.get_or_insert(now);
    }

    pub fn restored(&mut self) {
        self.since = None;
    }

    pub fn expired(&mut self, now: Instant) -> bool {
        let over = self.since.is_some_and(|since| now.duration_since(since) >= MINIMIZED_GRACE);
        if over {
            self.since = None;
        }
        over
    }
}

pub struct Lifecycle {
    _subscription: Subscription,
    lines: Receiver<String>,
}

fn is_relaunch(line: &str) -> bool {
    serde_json::from_str::<Value>(line).is_ok_and(|message| message["event"] == "relaunch")
}

pub fn watch() -> Option<Lifecycle> {
    let (subscription, lines) = luna::subscribe(REGISTER, "{}")?;
    Some(Lifecycle { _subscription: subscription, lines })
}

impl Lifecycle {
    pub fn relaunched(&self) -> bool {
        self.lines.try_iter().any(|line| is_relaunch(&line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grace_expires_once_after_two_minutes_minimised() {
        let start = Instant::now();
        let mut grace = Grace::default();
        grace.minimized(start);
        assert!(!grace.expired(start + Duration::from_secs(119)));
        assert!(grace.expired(start + Duration::from_secs(120)));
        assert!(!grace.expired(start + Duration::from_secs(300)));
    }

    #[test]
    fn restoring_cancels_the_grace() {
        let start = Instant::now();
        let mut grace = Grace::default();
        grace.minimized(start);
        grace.restored();
        assert!(!grace.expired(start + Duration::from_secs(500)));
    }

    #[test]
    fn only_the_relaunch_event_counts() {
        assert!(is_relaunch(r#"{"event":"relaunch","parameters":{}}"#));
        assert!(!is_relaunch(r#"{"returnValue":true,"event":"registered"}"#));
        assert!(!is_relaunch("garbage"));
    }
}
