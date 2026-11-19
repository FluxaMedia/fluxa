use std::cell::{Cell, RefCell};

use crate::gfx::Rect;
use crate::home::ArtworkPriority;

#[derive(Clone)]
pub(crate) struct Touch {
    pub url: Option<String>,
    pub size: [u32; 2],
    pub priority: ArtworkPriority,
    pub id: crate::gfx::TextureId,
}

#[derive(Default)]
pub(crate) struct Log {
    pub dirty: bool,
    pub touches: Vec<Touch>,
    pub registers: Vec<(u64, Rect)>,
    pub states: Vec<(String, Rect, u8)>,
    pub claims: Vec<(String, crate::skin::SkinTarget)>,
}

thread_local! {
    static STACK: RefCell<Vec<Log>> = const { RefCell::new(Vec::new()) };
    static EPOCH: Cell<u64> = const { Cell::new(0) };
}

pub(crate) fn begin() {
    STACK.with(|stack| stack.borrow_mut().push(Log::default()));
}

pub(crate) fn end() -> Log {
    STACK.with(|stack| stack.borrow_mut().pop().unwrap_or_default())
}

fn with(apply: impl FnOnce(&mut Log)) {
    STACK.with(|stack| {
        if let Some(log) = stack.borrow_mut().last_mut() {
            apply(log);
        }
    });
}

pub fn mark_dirty() {
    with(|log| log.dirty = true);
}

pub fn note_texture(
    url: Option<&str>,
    size: [u32; 2],
    priority: ArtworkPriority,
    id: crate::gfx::TextureId,
) {
    with(|log| {
        log.touches.push(Touch {
            url: url.map(str::to_owned),
            size,
            priority,
            id,
        })
    });
}

pub(crate) fn note_register(node: u64, rect: Rect) {
    with(|log| log.registers.push((node, rect)));
}

pub(crate) fn note_state(key: &str, rect: Rect, flags: u8) {
    with(|log| log.states.push((key.to_owned(), rect, flags)));
}

pub(crate) fn note_claim(key: &str, target: &crate::skin::SkinTarget) {
    with(|log| log.claims.push((key.to_owned(), target.clone())));
}

pub fn bump_epoch() {
    EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
}

pub(crate) fn epoch() -> u64 {
    EPOCH.with(Cell::get)
}
