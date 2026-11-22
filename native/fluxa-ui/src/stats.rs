use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use web_time::Instant;

pub static BUILD_NS: AtomicU64 = AtomicU64::new(0);
pub static LAYOUT_NS: AtomicU64 = AtomicU64::new(0);
pub static STYLE_NS: AtomicU64 = AtomicU64::new(0);
pub static STYLE_CALLS: AtomicU64 = AtomicU64::new(0);
pub static CLASS_MISSES: AtomicU64 = AtomicU64::new(0);
pub static SOLVES: AtomicU64 = AtomicU64::new(0);
pub static SPAN_CALLS: AtomicU64 = AtomicU64::new(0);
pub static OUTER_CALLS: AtomicU64 = AtomicU64::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("FLUXA_FRAME_STATS").is_ok_and(|value| value != "0"))
}

pub(crate) struct Timer(Option<Instant>);

pub(crate) fn start() -> Timer {
    Timer(enabled().then(Instant::now))
}

impl Timer {
    pub(crate) fn add(self, slot: &AtomicU64) {
        if let Some(started) = self.0 {
            slot.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
        }
    }
}

pub(crate) fn count(slot: &AtomicU64) {
    if enabled() {
        slot.fetch_add(1, Relaxed);
    }
}

pub fn take() -> [u64; 8] {
    [
        &BUILD_NS,
        &LAYOUT_NS,
        &STYLE_NS,
        &STYLE_CALLS,
        &CLASS_MISSES,
        &SOLVES,
        &SPAN_CALLS,
        &OUTER_CALLS,
    ]
    .map(|slot| slot.swap(0, Relaxed))
}
