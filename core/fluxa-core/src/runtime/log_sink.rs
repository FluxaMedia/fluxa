use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_BUFFERED: usize = 200;

static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn buffer() -> &'static Mutex<VecDeque<String>> {
    LOG_BUFFER.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub(crate) fn record(context: &str, detail: &str) {
    let mut buf = buffer()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if buf.len() >= MAX_BUFFERED {
        buf.pop_front();
    }
    buf.push_back(format!("{context}: {detail}"));
}

pub fn drain_core_log_json() -> String {
    let mut buf = buffer()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let lines: Vec<String> = buf.drain(..).collect();
    serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The buffer is process-global, so these assertions tolerate interleaving
    // with other tests touching it concurrently rather than asserting exact
    // contents.
    #[test]
    fn record_appends_and_drain_returns_it_as_json() {
        record("wire_test_log_sink", "boom-9f3c1a");
        let drained: Vec<String> = serde_json::from_str(&drain_core_log_json()).unwrap();
        assert!(drained.contains(&"wire_test_log_sink: boom-9f3c1a".to_string()));
    }

    #[test]
    fn buffer_never_exceeds_max_buffered() {
        for i in 0..(MAX_BUFFERED + 10) {
            record("wire_test_log_sink_cap", &i.to_string());
        }
        let drained: Vec<String> = serde_json::from_str(&drain_core_log_json()).unwrap();
        assert!(drained.len() <= MAX_BUFFERED);
    }
}
