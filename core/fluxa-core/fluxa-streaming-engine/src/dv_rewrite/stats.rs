use std::sync::atomic::{AtomicU32, Ordering};

static RPU_CONVERTED: AtomicU32 = AtomicU32::new(0);
static RPU_FAILED: AtomicU32 = AtomicU32::new(0);
static EL_DROPPED: AtomicU32 = AtomicU32::new(0);
static SEGMENTS: AtomicU32 = AtomicU32::new(0);

pub(crate) fn reset() {
    RPU_CONVERTED.store(0, Ordering::Relaxed);
    RPU_FAILED.store(0, Ordering::Relaxed);
    EL_DROPPED.store(0, Ordering::Relaxed);
    SEGMENTS.store(0, Ordering::Relaxed);
}

pub(crate) fn add(rpu_converted: u32, rpu_failed: u32, el_dropped: u32) {
    RPU_CONVERTED.fetch_add(rpu_converted, Ordering::Relaxed);
    RPU_FAILED.fetch_add(rpu_failed, Ordering::Relaxed);
    EL_DROPPED.fetch_add(el_dropped, Ordering::Relaxed);
    SEGMENTS.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn as_json() -> String {
    format!(
        "{{\"rpu_converted\":{},\"rpu_failed\":{},\"el_dropped\":{},\"segments\":{}}}",
        RPU_CONVERTED.load(Ordering::Relaxed),
        RPU_FAILED.load(Ordering::Relaxed),
        EL_DROPPED.load(Ordering::Relaxed),
        SEGMENTS.load(Ordering::Relaxed),
    )
}
