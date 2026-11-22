//! Element-scoped custom properties and derived interaction states.
//!
//! A `--name: value` declared in any rule applies to the elements that rule
//! matches and to their descendants (nearest declaration wins) and falls back
//! to the `:root` table. Declarations are interned once at parse time, so a
//! style carries a small `u32` handle and resolution only touches the table
//! when some rule actually declares a scoped property.

use super::{Color32, SCOPES, mix_colors};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

type Set = Arc<Vec<(String, String)>>;

static SETS: RwLock<Vec<Set>> = RwLock::new(Vec::new());
static ANY: AtomicBool = AtomicBool::new(false);

thread_local! {
    static CURRENT: Cell<u32> = const { Cell::new(0) };
}

/// Handle for a set of declarations; `0` is the empty set. Equal sets share a handle.
pub(super) fn intern(vars: Vec<(String, String)>) -> u32 {
    if vars.is_empty() {
        return 0;
    }
    let mut sets = SETS.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(found) = sets.iter().position(|set| **set == vars) {
        return found as u32 + 1;
    }
    sets.push(Arc::new(vars));
    ANY.store(true, Ordering::Relaxed);
    sets.len() as u32
}

fn set(id: u32) -> Option<Set> {
    let at = (id as usize).checked_sub(1)?;
    SETS.read().ok()?.get(at).cloned()
}

/// The declarations of several matched rules, later rules overriding earlier ones.
pub(super) fn merge(ids: impl Iterator<Item = u32>) -> u32 {
    let ids: Vec<u32> = ids.filter(|id| *id != 0).collect();
    match ids.as_slice() {
        [] => 0,
        [only] => *only,
        many => {
            let mut merged: Vec<(String, String)> = Vec::new();
            for set in many.iter().filter_map(|id| set(*id)) {
                for (name, value) in set.iter() {
                    match merged.iter_mut().find(|(known, _)| known == name) {
                        Some(slot) => slot.1.clone_from(value),
                        None => merged.push((name.clone(), value.clone())),
                    }
                }
            }
            intern(merged)
        }
    }
}

fn find(id: u32, name: &str) -> Option<String> {
    set(id)?
        .iter()
        .rev()
        .find(|(known, _)| known == name)
        .map(|(_, value)| value.clone())
}

/// The nearest scoped declaration of `--name`: the element being styled, then its ancestors.
pub(super) fn lookup(name: &str) -> Option<String> {
    if !ANY.load(Ordering::Relaxed) {
        return None;
    }
    let own = CURRENT.with(Cell::get);
    if own != 0
        && let Some(value) = find(own, name)
    {
        return Some(value);
    }
    SCOPES.with(|stack| {
        stack
            .borrow()
            .iter()
            .rev()
            .filter(|scope| scope.frame.vars != 0)
            .find_map(|scope| find(scope.frame.vars, name))
    })
}

/// Makes `id` the element's own declarations until dropped.
pub(super) struct Own(u32);

impl Own {
    pub(super) fn enter(id: u32) -> Self {
        Self(CURRENT.with(|slot| slot.replace(id)))
    }
}

impl Drop for Own {
    fn drop(&mut self) {
        CURRENT.with(|slot| slot.set(self.0));
    }
}

/// Hover look derived from a background: opaque fills dim to 85 % opacity,
/// translucent fills gain 8 % of their remaining alpha headroom.
pub(super) fn hover_fill(fill: Color32) -> Color32 {
    let alpha = fill.a();
    if alpha >= 230 {
        return mix_colors(fill, 0.85, Color32::TRANSPARENT);
    }
    if alpha == 0 {
        return fill;
    }
    let lifted = (f32::from(alpha) + (255.0 - f32::from(alpha)) * 0.08).round();
    let scale = lifted / f32::from(alpha);
    let [r, g, b, _] = fill.to_array().map(|c| (f32::from(c) * scale).round().min(255.0) as u8);
    Color32::from_rgba_premultiplied(r, g, b, lifted as u8)
}
