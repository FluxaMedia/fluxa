use super::helpers::{parse, str_field};
use serde_json::{Value, json};

const MAX_PROFILES: i64 = 6;
const PROFILE_COLORS: [&str; 16] = [
    "#1E88E5", "#E53935", "#43A047", "#FB8C00", "#8E24AA", "#00ACC1", "#F4511E", "#3949AB",
    "#C0CA33", "#D81B60", "#00897B", "#5E35B1", "#7CB342", "#039BE5", "#FFB300", "#6D4C41",
];
const AVATAR_STORAGE_BASE: &str = "https://api.nuvio.tv/storage/v1/object/public/avatars/";
fn avatar_url_for(profile: &Value, avatar_catalog: &[Value]) -> Option<String> {
    if let Some(url) = str_field(profile, "avatar_url").filter(|s| !s.is_empty()) {
        return Some(url.to_string());
    }
    let avatar_id = profile.get("avatar_id").filter(|v| !v.is_null())?;
    let entry = avatar_catalog
        .iter()
        .find(|a| a.get("id") == Some(avatar_id))?;
    let storage_path = str_field(entry, "storage_path").filter(|s| !s.is_empty())?;
    Some(format!("{AVATAR_STORAGE_BASE}{storage_path}"))
}

pub(crate) fn avatar_catalog_pack_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let base = str_field(&args, "storageBase")
        .map(str::to_owned)
        .unwrap_or_else(|| AVATAR_STORAGE_BASE.to_owned());
    let mut items: Vec<&Value> = args
        .get("catalog")
        .and_then(Value::as_array)?
        .iter()
        .filter(|item| {
            item.get("is_active")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        })
        .collect();
    items.sort_by_key(|item| {
        (
            str_field(item, "category").unwrap_or_default().to_owned(),
            item.get("sort_order").and_then(Value::as_i64).unwrap_or(0),
        )
    });
    let mut packs: Vec<(String, Vec<Value>)> = Vec::new();
    for item in items {
        let Some(path) = str_field(item, "storage_path").filter(|path| !path.is_empty()) else {
            continue;
        };
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_owned()
        } else {
            format!("{base}{path}")
        };
        let category = str_field(item, "category").unwrap_or_default().to_owned();
        let avatar = json!({
            "name": str_field(item, "display_name").unwrap_or_default(),
            "url": url,
        });
        match packs.last_mut() {
            Some((last, avatars)) if *last == category => avatars.push(avatar),
            _ => packs.push((category, vec![avatar])),
        }
    }
    let packs: Vec<Value> = packs
        .into_iter()
        .map(|(category, avatars)| {
            let mut title = category.clone();
            if let Some(first) = title.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            json!({"id": format!("nuvio:{category}"), "title": title, "avatars": avatars})
        })
        .collect();
    serde_json::to_string(&json!({ "packs": packs })).ok()
}

fn pin_enabled(source: &Value) -> bool {
    source
        .get("pin_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn apply_remote_profiles_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let session = args.get("sessionProfile")?;
    let remote = args.get("nuvioProfiles").and_then(Value::as_array)?;
    let avatar_catalog = args
        .get("avatarCatalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut profiles = args.get("profiles").and_then(Value::as_array)?.clone();
    let account = |profile: &Value| {
        ["nuvioUserId", "nuvioEmail", "email"]
            .iter()
            .find_map(|key| {
                str_field(profile, key)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
            })
    };
    let owner = account(session)?;
    let mut changed = false;
    for profile in &mut profiles {
        if account(profile).as_deref() != Some(owner.as_str()) {
            continue;
        }
        let index = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .unwrap_or(1);
        let Some(source) = remote
            .iter()
            .find(|entry| entry.get("profile_index").and_then(Value::as_i64) == Some(index))
        else {
            continue;
        };
        let Some(fields) = profile.as_object_mut() else {
            continue;
        };
        let mut set = |key: &str, value: Value| {
            if fields.get(key) != Some(&value) {
                fields.insert(key.into(), value);
                changed = true;
            }
        };
        if let Some(name) = str_field(source, "name").filter(|s| !s.trim().is_empty()) {
            set("name", Value::String(name.trim().to_owned()));
        }
        if let Some(url) = avatar_url_for(source, &avatar_catalog) {
            set("avatarUrl", Value::String(url));
        }
        set("nuvioProfileIndex", json!(index));
        set("nuvioPinEnabled", json!(pin_enabled(source)));
        set(
            "nuvioUpdatedAt",
            json!(str_field(source, "updated_at").unwrap_or_default()),
        );
        let updated_at = str_field(source, "updated_at").unwrap_or_default();
        let cached_at = fields.get("nuvioPinCachedAt").and_then(Value::as_str);
        if !pin_enabled(source) || cached_at.is_some_and(|cached| cached != updated_at) {
            changed |= super::pin::clear_cache_fields(fields);
        }
    }
    let known: Vec<i64> = profiles
        .iter()
        .filter(|profile| account(profile).as_deref() == Some(owner.as_str()))
        .map(|profile| {
            profile
                .get("nuvioProfileIndex")
                .and_then(Value::as_i64)
                .unwrap_or(1)
        })
        .collect();
    let session_id = str_field(session, "id").unwrap_or_default().to_owned();
    for source in remote {
        let Some(index) = source
            .get("profile_index")
            .and_then(Value::as_i64)
            .filter(|index| *index > 0 && !known.contains(index))
        else {
            continue;
        };
        let mut profile = session.clone();
        let Some(fields) = profile.as_object_mut() else {
            continue;
        };
        fields.insert("id".into(), json!(format!("{session_id}-{index}")));
        fields.insert("nuvioProfileIndex".into(), json!(index));
        fields.insert(
            "name".into(),
            json!(str_field(source, "name").unwrap_or_default().trim()),
        );
        if let Some(url) = avatar_url_for(source, &avatar_catalog) {
            fields.insert("avatarUrl".into(), json!(url));
        }
        fields.insert("nuvioPinEnabled".into(), json!(pin_enabled(source)));
        fields.insert(
            "nuvioUpdatedAt".into(),
            json!(str_field(source, "updated_at").unwrap_or_default()),
        );
        super::pin::clear_cache_fields(fields);
        profiles.push(profile);
        changed = true;
    }
    changed.then(|| Value::Array(profiles).to_string())
}

fn valid_avatar_url(url: &str) -> bool {
    url.len() <= 2048
        && !url.chars().any(char::is_whitespace)
        && (url.starts_with("https://") || url.starts_with("http://"))
}

fn catalog_avatar_id(url: &str, avatar_catalog: &[Value]) -> Option<Value> {
    avatar_catalog.iter().find_map(|entry| {
        let path = str_field(entry, "storage_path").filter(|path| !path.is_empty())?;
        let full = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_owned()
        } else {
            format!("{AVATAR_STORAGE_BASE}{path}")
        };
        (full == url).then(|| entry.get("id").cloned()).flatten()
    })
}

pub(crate) fn profile_push_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let local = args.get("profile")?;
    let remote = args.get("nuvioProfiles").and_then(Value::as_array)?;
    let taken: Vec<i64> = remote
        .iter()
        .filter_map(|source| source.get("profile_index").and_then(Value::as_i64))
        .collect();
    let index = match args.get("profileIndex").and_then(Value::as_i64) {
        Some(index) if index > 0 => index,
        _ => (1..=MAX_PROFILES).find(|index| !taken.contains(index))?,
    };
    let avatar_catalog = args
        .get("avatarCatalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut rows: Vec<Value> = remote
        .iter()
        .filter_map(|source| {
            let row_index = source.get("profile_index").and_then(Value::as_i64)?;
            Some(json!({
                "profile_index": row_index,
                "name": str_field(source, "name").unwrap_or_default(),
                "avatar_color_hex": str_field(source, "avatar_color_hex").unwrap_or("#1E88E5"),
                "uses_primary_addons": source.get("uses_primary_addons").and_then(Value::as_bool).unwrap_or(false),
                "uses_primary_plugins": source.get("uses_primary_plugins").and_then(Value::as_bool).unwrap_or(false),
                "avatar_id": source.get("avatar_id").cloned().unwrap_or(Value::Null),
                "avatar_url": source.get("avatar_url").cloned().unwrap_or(Value::Null),
                "profile_background_id": source.get("profile_background_id").cloned().unwrap_or(Value::Null),
                "profile_background_url": source.get("profile_background_url").cloned().unwrap_or(Value::Null),
            }))
        })
        .collect();
    if !rows
        .iter()
        .any(|row| row.get("profile_index").and_then(Value::as_i64) == Some(index))
    {
        let color = PROFILE_COLORS[(index as usize - 1) % PROFILE_COLORS.len()];
        rows.push(json!({"profile_index": index, "avatar_color_hex": color}));
    }
    let source = remote
        .iter()
        .find(|source| source.get("profile_index").and_then(Value::as_i64) == Some(index));
    let row = rows
        .iter_mut()
        .find(|row| row.get("profile_index").and_then(Value::as_i64) == Some(index))?
        .as_object_mut()?;
    if let Some(name) = str_field(local, "name").filter(|name| !name.trim().is_empty()) {
        row.insert("name".into(), json!(name.trim()));
    }
    if index != 1 {
        for (local_key, key) in [
            ("usesPrimaryAddons", "uses_primary_addons"),
            ("usesPrimaryPlugins", "uses_primary_plugins"),
        ] {
            if let Some(flag) = local.get(local_key).and_then(Value::as_bool) {
                row.insert(key.into(), json!(flag));
            }
        }
    }
    let remote_url = source.and_then(|source| avatar_url_for(source, &avatar_catalog));
    if let Some(url) = str_field(local, "avatarUrl")
        .map(str::trim)
        .filter(|url| valid_avatar_url(url))
        .filter(|url| remote_url.as_deref() != Some(*url))
    {
        match catalog_avatar_id(url, &avatar_catalog) {
            Some(id) => {
                row.insert("avatar_id".into(), id);
                row.insert("avatar_url".into(), Value::Null);
            }
            None => {
                row.insert("avatar_id".into(), Value::Null);
                row.insert("avatar_url".into(), json!(url));
            }
        }
    }
    rows.sort_by_key(|row| {
        row.get("profile_index")
            .and_then(Value::as_i64)
            .unwrap_or(0)
    });
    Some(
        json!({
            "profileIndex": index,
            "body": {"p_client_max_profiles": MAX_PROFILES, "p_profiles": rows},
        })
        .to_string(),
    )
}

pub(crate) fn effective_profile_scopes_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let profile_index = args
        .get("profileIndex")
        .and_then(Value::as_i64)
        .filter(|index| *index > 0)
        .unwrap_or(1);
    let profiles = args.get("profiles").and_then(Value::as_array)?;
    let profile = profiles.iter().find(|profile| {
        profile
            .get("profile_index")
            .or_else(|| profile.get("profileIndex"))
            .and_then(Value::as_i64)
            == Some(profile_index)
    });
    let uses_primary = |snake: &str, camel: &str| {
        profile
            .and_then(|value| value.get(snake).or_else(|| value.get(camel)))
            .and_then(Value::as_bool)
            == Some(true)
    };
    Some(
        json!({
            "addons": if profile_index != 1 && uses_primary("uses_primary_addons", "usesPrimaryAddons") { 1 } else { profile_index },
            "plugins": if profile_index != 1 && uses_primary("uses_primary_plugins", "usesPrimaryPlugins") { 1 } else { profile_index },
        })
        .to_string(),
    )
}
