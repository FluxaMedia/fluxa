use std::sync::mpsc::Receiver;

use serde_json::Value;

use crate::luna::{self, Subscription};

const REGISTER: &str = "luna://com.webos.service.applicationmanager/registerApp";

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
    fn only_the_relaunch_event_counts() {
        assert!(is_relaunch(r#"{"event":"relaunch","parameters":{}}"#));
        assert!(!is_relaunch(r#"{"returnValue":true,"event":"registered"}"#));
        assert!(!is_relaunch("garbage"));
    }
}
