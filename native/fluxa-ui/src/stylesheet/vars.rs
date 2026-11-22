use super::{Query, css};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

const SHEET: &str = include_str!("../../../../shared/skins/default/style.css");

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
    for (key, vars) in super::defaults::layers() {
        let queries = match key {
            Some(key) => vec![Query::parse(&format!("@{key}"))?],
            None => Vec::new(),
        };
        table.layers.push(Layer { queries, vars });
    }
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
    let table = if extra.trim() == SHEET.trim() {
        build(&[SHEET])?
    } else {
        build(&[SHEET, extra])?
    };
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
    let table = Arc::new(build(&[SHEET]).expect("bundled fluxa.css"));
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
    let values = Arc::new(expand_references(values));
    *slot = Some(Resolved {
        generation,
        key,
        values: values.clone(),
    });
    values
}

fn expand_references(
    mut values: HashMap<String, String, crate::fast::Build>,
) -> HashMap<String, String, crate::fast::Build> {
    for _ in 0..4 {
        let snapshot = values.clone();
        let mut changed = false;
        for value in values.values_mut() {
            let Some(start) = value.find("var(--") else {
                continue;
            };
            let Some(end) = value[start..].find(')') else {
                continue;
            };
            let name = &value[start + 6..start + end];
            if let Some(found) = snapshot.get(name) {
                value.replace_range(start..start + end + 1, found);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    values
}

pub(crate) fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn color(name: &str) -> Option<crate::gfx::Color32> {
    let values = resolved();
    let value = values.get(name)?;
    if value.trim().starts_with("color-mix(") {
        return super::parse_paint(value.trim()).map(|paint| super::resolve_paint(&paint));
    }
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
    let values = resolved();
    let value = values.get(name);
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
    let values = resolved();
    let value = values.get(name);
    debug_assert!(value.is_some(), "missing metric --{name}");
    value.map_or(0.0, |value| clamp(value, basis))
}

fn clamp(value: &str, basis: f32) -> f32 {
    let Some(inner) = value
        .trim()
        .strip_prefix("clamp(")
        .and_then(|v| v.strip_suffix(')'))
    else {
        return 0.0;
    };
    let mut parts = inner.split(',').map(|part| sum(part, basis));
    let (Some(min), Some(mid), Some(max)) = (parts.next(), parts.next(), parts.next()) else {
        return 0.0;
    };
    mid.clamp(min, max.max(min))
}

fn sum(expression: &str, basis: f32) -> f32 {
    let mut total = 0.0;
    let mut sign = 1.0;
    for term in expression.split_whitespace() {
        match term {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            _ => {
                total += sign
                    * match term.strip_suffix('%') {
                        Some(percent) => percent.parse::<f32>().unwrap_or(0.0) * basis / 100.0,
                        None => term.trim_end_matches("px").parse::<f32>().unwrap_or(0.0),
                    };
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{Query, build, clamp};

    #[test]
    fn clamp_follows_basis_between_bounds() {
        assert_eq!(clamp("clamp(14px, 13%, 28px)", 100.0), 14.0);
        assert_eq!(clamp("clamp(14px, 13%, 28px)", 150.0), 19.5);
        assert_eq!(clamp("clamp(14px, 13%, 28px)", 900.0), 28.0);
        assert_eq!(clamp("clamp(0px, 50% + 20px, 500px)", 100.0), 70.0);
    }

    #[test]
    fn root_font_size_is_a_root_layer_and_html_is_an_alias() {
        let table = build(&[
            ":root { font-size: 18px; }\n@media (size: compact) { html { font-size: 12px; } }",
        ])
        .expect("table");
        let sizes: Vec<_> = table
            .layers
            .iter()
            .flat_map(|layer| {
                layer
                    .vars
                    .iter()
                    .filter(|(name, _)| name == "root-font-size")
                    .map(|(_, value)| (layer.queries.first().cloned(), value.as_str()))
            })
            .collect();
        assert_eq!(sizes.first(), Some(&(None, "16px")));
        assert!(sizes.contains(&(None, "18px")));
        assert_eq!(
            sizes.last(),
            Some(&(Some(Query::Window("compact".to_owned())), "12px"))
        );
    }

    #[test]
    fn skin_declarations_override_engine_defaults() {
        let table = build(&[":root { --button-border: 30px; }"]).expect("table");
        let last = |name: &str| {
            table
                .layers
                .iter()
                .flat_map(|layer| layer.vars.iter())
                .filter(|(key, _)| key == name)
                .last()
                .map(|(_, value)| value.clone())
        };
        assert_eq!(last("button-border").as_deref(), Some("30px"));
        assert_eq!(last("pill-ratio").as_deref(), Some("0.5"));
    }
}
