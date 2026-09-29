use serde_json::{Value, json};

pub(crate) struct Provider {
    pub id: &'static str,
    pub token_field: &'static str,
    pub refresh_fields: Option<(&'static str, &'static str)>,
    pub capabilities: &'static [&'static str],
    pub calendar_plan: Option<fn(&str) -> Option<String>>,
    pub snapshot: Option<fn(&Value, &Value) -> Value>,
    pub library_requests: Option<fn(&Value) -> Option<Vec<Value>>>,
}

pub(crate) const PROVIDERS: &[Provider] = &[
    Provider {
        id: "trakt",
        token_field: "traktAccessToken",
        refresh_fields: Some(("traktRefreshToken", "traktTokenExpiresAt")),
        capabilities: &[
            "device",
            "library",
            "scrobble",
            "watchedSync",
            "watchlistWrite",
            "calendar",
        ],
        calendar_plan: Some(super::provider::trakt::trakt_calendar_plan_json),
        snapshot: Some(super::provider::trakt::trakt_library_snapshot),
        library_requests: Some(super::provider::trakt::trakt_library_requests),
    },
    Provider {
        id: "simkl",
        token_field: "simklAccessToken",
        refresh_fields: Some(("simklRefreshToken", "simklTokenExpiresAt")),
        capabilities: &[
            "device",
            "browser",
            "library",
            "scrobble",
            "watchedSync",
            "watchlistWrite",
            "calendar",
        ],
        calendar_plan: None,
        snapshot: Some(super::provider::simkl::simkl_library_snapshot),
        library_requests: None,
    },
    Provider {
        id: "mdblist",
        token_field: "mdblistAccessToken",
        refresh_fields: Some(("mdblistRefreshToken", "mdblistTokenExpiresAt")),
        capabilities: &[
            "device",
            "library",
            "scrobble",
            "watchedSync",
            "watchlistWrite",
            "calendar",
        ],
        calendar_plan: Some(super::provider::mdblist::mdblist_calendar_plan_json),
        snapshot: Some(super::provider::mdblist::mdblist_library_snapshot),
        library_requests: Some(super::provider::mdblist::mdblist_library_requests),
    },
    Provider {
        id: "anilist",
        token_field: "anilistAccessToken",
        refresh_fields: None,
        capabilities: &[
            "browser",
            "library",
            "watchedSync",
            "watchlistWrite",
            "calendar",
        ],
        calendar_plan: Some(super::provider::anilist::anilist_calendar_plan_json),
        snapshot: Some(super::provider::anilist::anilist_library_snapshot),
        library_requests: Some(super::provider::anilist::anilist_library_requests),
    },
];

pub(crate) fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|provider| provider.id == id)
}

impl Provider {
    pub fn can(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability)
    }

    fn connected(&self, profile: &Value) -> bool {
        let set = |key: &str| {
            profile
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        };
        set(self.token_field) || (self.id == "mdblist" && set("mdblistApiKey"))
    }
}

pub(crate) fn provider_registry_json() -> String {
    let providers: Vec<Value> = PROVIDERS
        .iter()
        .map(|provider| {
            json!({
                "id": provider.id,
                "tokenField": provider.token_field,
                "refreshFields": provider.refresh_fields.map(|(refresh, expires)| json!([refresh, expires])),
                "capabilities": provider.capabilities,
            })
        })
        .collect();
    json!(providers).to_string()
}

pub(crate) fn connected_providers_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let profile = args.get("profile").unwrap_or(&Value::Null);
    let capability = args.get("capability").and_then(Value::as_str);
    let ids: Vec<&str> = PROVIDERS
        .iter()
        .filter(|provider| capability.is_none_or(|capability| provider.can(capability)))
        .filter(|provider| provider.connected(profile))
        .map(|provider| provider.id)
        .collect();
    serde_json::to_string(&ids).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_connected_providers_with_the_capability_are_returned() {
        let profile =
            json!({"traktAccessToken": "t", "anilistAccessToken": "a", "simklAccessToken": ""});
        let scrobble = connected_providers_json(
            &json!({"profile": profile, "capability": "scrobble"}).to_string(),
        );
        assert_eq!(scrobble.as_deref(), Some(r#"["trakt"]"#));
        let sync = connected_providers_json(
            &json!({"profile": profile, "capability": "watchedSync"}).to_string(),
        );
        assert_eq!(sync.as_deref(), Some(r#"["trakt","anilist"]"#));
    }

    #[test]
    fn mdblist_api_key_counts_as_connected() {
        let profile = json!({"mdblistApiKey": "k"});
        let ids = connected_providers_json(&json!({"profile": profile}).to_string());
        assert_eq!(ids.as_deref(), Some(r#"["mdblist"]"#));
    }
}
