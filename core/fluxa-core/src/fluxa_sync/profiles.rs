use serde_json::{Map, Value, json};

fn remote_settings(profile: &Value) -> Map<String, Value> {
    let mut settings = Map::new();
    for field in [
        "color",
        "isAnonymous",
        "usesPrimaryAddons",
        "usesPrimaryPlugins",
    ] {
        if let Some(value) = profile.get(field).filter(|value| !value.is_null()) {
            settings.insert(field.to_string(), value.clone());
        }
    }
    if let Some(id) = profile.get("id").and_then(Value::as_str) {
        settings.insert("localId".into(), json!(id));
    }
    settings
}

fn remote_body(profile: &Value) -> Value {
    json!({
        "name": profile.get("name").and_then(Value::as_str).unwrap_or("Profile"),
        "avatar": profile.get("avatarUrl").cloned().unwrap_or(Value::Null),
        "settings": Value::Object(remote_settings(profile)),
    })
}

fn linked_remote<'a>(local: &Value, remote: &'a [Value]) -> Option<&'a Value> {
    let linked = local.get("fluxaProfileId").and_then(Value::as_str);
    let local_id = local.get("id").and_then(Value::as_str);
    remote.iter().find(|candidate| {
        let id = candidate.get("id").and_then(Value::as_str);
        let tagged = candidate
            .get("settings")
            .and_then(|settings| settings.get("localId"))
            .and_then(Value::as_str);
        (linked.is_some() && id == linked) || (tagged.is_some() && tagged == local_id)
    })
}

fn merge_into_local(local: &Value, remote: &Value) -> Value {
    let mut merged = local.as_object().cloned().unwrap_or_default();
    if let Some(id) = remote.get("id").and_then(Value::as_str) {
        merged.insert("fluxaProfileId".into(), json!(id));
    }
    if let Some(name) = remote.get("name").and_then(Value::as_str) {
        merged.insert("name".into(), json!(name));
    }
    match remote.get("avatar") {
        Some(avatar) if !avatar.is_null() => {
            merged.insert("avatarUrl".into(), avatar.clone());
        }
        _ => {}
    }
    if let Some(settings) = remote.get("settings").and_then(Value::as_object) {
        for field in [
            "color",
            "isAnonymous",
            "usesPrimaryAddons",
            "usesPrimaryPlugins",
        ] {
            if let Some(value) = settings.get(field) {
                merged.insert(field.to_string(), value.clone());
            }
        }
    }
    Value::Object(merged)
}

fn local_from_remote(remote: &Value) -> Value {
    let mut local = Map::new();
    let settings = remote.get("settings").cloned().unwrap_or(Value::Null);
    let local_id = settings
        .get("localId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            remote
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    local.insert("id".into(), json!(local_id));
    local.insert(
        "fluxaProfileId".into(),
        remote.get("id").cloned().unwrap_or(Value::Null),
    );
    local.insert(
        "name".into(),
        remote.get("name").cloned().unwrap_or(Value::Null),
    );
    if let Some(avatar) = remote.get("avatar").filter(|value| !value.is_null()) {
        local.insert("avatarUrl".into(), avatar.clone());
    }
    if let Some(fields) = settings.as_object() {
        for field in [
            "color",
            "isAnonymous",
            "usesPrimaryAddons",
            "usesPrimaryPlugins",
        ] {
            if let Some(value) = fields.get(field) {
                local.insert(field.to_string(), value.clone());
            }
        }
    }
    Value::Object(local)
}

pub(crate) fn profile_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let local = args
        .get("local")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let remote = args
        .get("remote")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut creates: Vec<Value> = Vec::new();
    let mut updates: Vec<Value> = Vec::new();
    let mut merged: Vec<Value> = Vec::new();
    let mut matched: Vec<String> = Vec::new();

    for profile in &local {
        match linked_remote(profile, &remote) {
            Some(counterpart) => {
                if let Some(id) = counterpart.get("id").and_then(Value::as_str) {
                    matched.push(id.to_string());
                    let body = remote_body(profile);
                    let unchanged = counterpart.get("name") == body.get("name")
                        && counterpart.get("avatar") == body.get("avatar")
                        && counterpart.get("settings") == body.get("settings");
                    if !unchanged {
                        updates.push(json!({ "id": id, "body": body }));
                    }
                }
                merged.push(merge_into_local(profile, counterpart));
            }
            None => {
                creates.push(json!({
                    "localId": profile.get("id").cloned().unwrap_or(Value::Null),
                    "body": remote_body(profile),
                }));
                merged.push(profile.clone());
            }
        }
    }

    for counterpart in &remote {
        let id = counterpart.get("id").and_then(Value::as_str).unwrap_or("");
        if matched.iter().any(|seen| seen == id) {
            continue;
        }
        merged.push(local_from_remote(counterpart));
    }

    serde_json::to_string(&json!({
        "creates": creates,
        "updates": updates,
        "profiles": merged,
    }))
    .ok()
}
