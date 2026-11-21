use super::{Query, css};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

const METRICS: &str = include_str!("../../../../shared/contracts/ui-metrics.css");
const THEME: &str = include_str!("../../../../shared/contracts/ui-theme.css");

struct Layer {
    queries: Vec<Query>,
    vars: Vec<(String, String)>,
}

#[derive(Default)]
struct Table {
    layers: Vec<Layer>,
}

static TABLE: RwLock<Option<Arc<Table>>> = RwLock::new(None);
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Default)]
struct Resolved {
    generation: u64,
    key: Option<crate::metrics::MetricsKey>,
    values: Arc<HashMap<String, String, crate::fast::Build>>,
}

static RESOLVED: Mutex<Option<Resolved>> = Mutex::new(None);

fn build(texts: &[&str]) -> Result<Table, String> {
    let mut table = Table::default();
    for text in texts {
        for (keys, vars) in css::root_layers(text)? {
            let queries = keys
                .iter()
                .map(|key| Query::parse(&format!("@{key}")))
                .collect::<Result<Vec<_>, _>>()?;
            table.layers.push(Layer { queries, vars });
        }
    }
    Ok(table)
}

pub(crate) fn load(extra: &str) -> Result<(), String> {
    let table = build(&[METRICS, THEME, extra])?;
    if let Ok(mut slot) = TABLE.write() {
        *slot = Some(Arc::new(table));
    }
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

fn table() -> Arc<Table> {
    if let Some(table) = TABLE.read().ok().and_then(|slot| slot.clone()) {
        return table;
    }
    let table = Arc::new(build(&[METRICS, THEME]).expect("bundled ui-metrics"));
    if let Ok(mut slot) = TABLE.write() {
        *slot = Some(table.clone());
    }
    table
}

fn resolved() -> Arc<HashMap<String, String, crate::fast::Build>> {
    let generation = GENERATION.load(std::sync::atomic::Ordering::Relaxed);
    let key = crate::metrics::active_key();
    let mut slot = RESOLVED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(hit) = slot.as_ref()
        && hit.generation == generation
        && hit.key == key
    {
        return hit.values.clone();
    }
    let table = table();
    let mut values: HashMap<String, String, crate::fast::Build> = Default::default();
    for layer in &table.layers {
        if layer.queries.is_empty() || layer.queries.iter().any(Query::matches) {
            for (name, value) in &layer.vars {
                values.insert(name.clone(), value.clone());
            }
        }
    }
    for (name, value) in fluxa_theme::user::active_vars() {
        values.insert(name, value);
    }
    let values = Arc::new(values);
    *slot = Some(Resolved {
        generation,
        key,
        values: values.clone(),
    });
    values
}

pub(crate) fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn color(name: &str) -> Option<crate::gfx::Color32> {
    let value = raw(name)?;
    let color = fluxa_theme::theme::parse_hex_color(name, value.trim()).ok()?;
    Some(crate::gfx::Color32::from_rgba_unmultiplied(
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        (color.a * 255.0).round() as u8,
    ))
}

pub(crate) fn paint(name: &str) -> crate::gfx::Color32 {
    color(name).unwrap_or(crate::gfx::Color32::TRANSPARENT)
}

pub(crate) fn raw(name: &str) -> Option<String> {
    resolved().get(name).cloned()
}

pub(crate) fn px(name: &str) -> f32 {
    let value = raw(name);
    debug_assert!(value.is_some(), "missing metric --{name}");
    value
        .and_then(|value| value.trim().trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or(0.0)
}

pub(crate) fn byte(name: &str) -> u8 {
    px(name).round().clamp(0.0, 255.0) as u8
}

pub(crate) fn signed(name: &str) -> i8 {
    px(name).round().clamp(-128.0, 127.0) as i8
}

pub(crate) fn scaled(name: &str, basis: f32) -> f32 {
    let ratio = px(&format!("{name}-ratio"));
    let min = px(&format!("{name}-min"));
    let max = px(&format!("{name}-max"));
    let offset = px(&format!("{name}-offset"));
    (basis * ratio + offset).clamp(min, max)
}
