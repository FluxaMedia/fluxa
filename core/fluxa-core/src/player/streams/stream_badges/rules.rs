use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

pub const MAX_PACKS: usize = 3;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Filter {
    pub id: String,
    pub group_id: Option<String>,
    pub name: String,
    pub pattern: String,
    #[serde(rename = "imageURL", alias = "imageUrl")]
    pub image_url: Option<String>,
    #[serde(default = "enabled")]
    pub is_enabled: bool,
    pub tag_color: Option<String>,
    pub tag_style: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
}

fn enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub source_url: Option<String>,
    #[serde(default = "enabled")]
    pub is_active: bool,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rules {
    pub custom: Vec<Filter>,
    pub packs: Vec<Pack>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BadgeLook {
    pub fill: Option<u32>,
    pub text: Option<u32>,
    pub border: Option<u32>,
    pub outline: bool,
}

static COMPILED: Mutex<Option<HashMap<String, Option<Regex>>>> = Mutex::new(None);

pub fn compile(pattern: &str) -> Option<Regex> {
    let mut cache = COMPILED.lock().ok()?;
    let cache = cache.get_or_insert_with(HashMap::new);
    if cache.len() > 512 {
        cache.clear();
    }
    cache
        .entry(pattern.to_owned())
        .or_insert_with(|| Regex::new(&format!("(?i){pattern}")).ok())
        .clone()
}

pub fn valid_pattern(pattern: &str) -> bool {
    !pattern.trim().is_empty() && compile(pattern).is_some()
}

pub fn parse_color(value: &str) -> Option<u32> {
    let hex = value.trim().trim_start_matches('#');
    let (rgb, alpha) = match hex.len() {
        6 => (u32::from_str_radix(hex, 16).ok()?, 0xff),
        8 => {
            let argb = u32::from_str_radix(hex, 16).ok()?;
            (argb & 0x00ff_ffff, argb >> 24)
        }
        _ => return None,
    };
    Some(rgb << 8 | alpha)
}

impl Filter {
    pub fn look(&self) -> BadgeLook {
        let color = |value: &Option<String>| value.as_deref().and_then(parse_color);
        let style = self.tag_style.as_deref().unwrap_or_default();
        BadgeLook {
            fill: color(&self.tag_color),
            text: color(&self.text_color),
            border: color(&self.border_color),
            outline: style.eq_ignore_ascii_case("outline"),
        }
    }

    pub fn matches(&self, text: &str) -> bool {
        self.is_enabled && compile(&self.pattern).is_some_and(|regex| regex.is_match(text))
    }
}

impl Rules {
    pub fn from_value(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn active_filters(&self) -> impl Iterator<Item = &Filter> {
        self.packs
            .iter()
            .filter(|pack| pack.is_active)
            .flat_map(|pack| pack.filters.iter())
            .chain(self.custom.iter())
    }

    pub fn upsert_custom(&mut self, mut filter: Filter) -> bool {
        filter.name = filter.name.trim().to_owned();
        if filter.name.is_empty() || !valid_pattern(&filter.pattern) {
            return false;
        }
        if filter.id.is_empty() {
            filter.id = format!(
                "custom-{}",
                fingerprint(&format!("{}{}", filter.name, filter.pattern))
            );
            while self.custom.iter().any(|existing| existing.id == filter.id) {
                filter.id.push('x');
            }
        }
        match self
            .custom
            .iter_mut()
            .find(|existing| existing.id == filter.id)
        {
            Some(existing) => *existing = filter,
            None => self.custom.push(filter),
        }
        true
    }

    pub fn remove_custom(&mut self, id: &str) {
        self.custom.retain(|filter| filter.id != id);
    }

    pub fn toggle_custom(&mut self, id: &str) {
        if let Some(filter) = self.custom.iter_mut().find(|filter| filter.id == id) {
            filter.is_enabled = !filter.is_enabled;
        }
    }

    pub fn add_pack(&mut self, pack: Pack) -> bool {
        let existing = self.packs.iter().position(|current| {
            current.id == pack.id
                || (pack.source_url.is_some() && current.source_url == pack.source_url)
        });
        match existing {
            Some(index) => self.packs[index] = pack,
            None if self.packs.len() >= MAX_PACKS => return false,
            None => self.packs.push(pack),
        }
        true
    }

    pub fn remove_pack(&mut self, id: &str) {
        self.packs.retain(|pack| pack.id != id);
    }

    pub fn toggle_pack(&mut self, id: &str) {
        if let Some(pack) = self.packs.iter_mut().find(|pack| pack.id == id) {
            pack.is_active = !pack.is_active;
        }
    }
}

pub fn parse_pack(text: &str, source_url: Option<&str>) -> Result<Pack, String> {
    let value: Value = serde_json::from_str(text.trim()).map_err(|error| error.to_string())?;
    let (name, filters) = match &value {
        Value::Array(_) => (None, value.clone()),
        Value::Object(map) => (
            map.get("name").and_then(Value::as_str).map(str::to_owned),
            map.get("filters").cloned().unwrap_or(Value::Null),
        ),
        _ => return Err("not a badge pack".to_owned()),
    };
    let filters: Vec<Filter> =
        serde_json::from_value(filters).map_err(|error| error.to_string())?;
    let filters: Vec<Filter> = filters
        .into_iter()
        .enumerate()
        .filter(|(_, filter)| valid_pattern(&filter.pattern) && !filter.name.trim().is_empty())
        .map(|(index, mut filter)| {
            if filter.id.is_empty() {
                filter.id = format!("f{index}");
            }
            filter
        })
        .collect();
    if filters.is_empty() {
        return Err("no usable badges in this pack".to_owned());
    }
    let id = fingerprint(source_url.unwrap_or(text));
    let name = name
        .filter(|name| !name.trim().is_empty())
        .or_else(|| source_url.and_then(host))
        .unwrap_or_else(|| "Imported badges".to_owned());
    Ok(Pack {
        id,
        name,
        source_url: source_url.map(str::to_owned),
        is_active: true,
        filters,
    })
}

fn host(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    Some(rest.split('/').next()?.to_owned()).filter(|host| !host.is_empty())
}

fn fingerprint(text: &str) -> String {
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}
