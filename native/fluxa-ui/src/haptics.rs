use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy)]
pub enum Haptic {
    LongPress = 1,
    ToggleOn = 2,
    ToggleOff = 3,
}

static PENDING: AtomicU8 = AtomicU8::new(0);

pub fn buzz(haptic: Haptic) {
    PENDING.store(haptic as u8, Ordering::Relaxed);
}

pub fn take_haptic() -> u8 {
    PENDING.swap(0, Ordering::Relaxed)
}
