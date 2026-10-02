use std::sync::{Condvar, Mutex};

static HELD: Mutex<bool> = Mutex::new(false);
static RELEASED: Condvar = Condvar::new();

pub fn lock() {
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    while *held {
        held = RELEASED.wait(held).unwrap_or_else(|e| e.into_inner());
    }
    *held = true;
}

pub fn unlock() {
    *HELD.lock().unwrap_or_else(|e| e.into_inner()) = false;
    RELEASED.notify_one();
}

pub struct QueueGuard;

pub fn guard() -> QueueGuard {
    lock();
    QueueGuard
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        unlock();
    }
}
