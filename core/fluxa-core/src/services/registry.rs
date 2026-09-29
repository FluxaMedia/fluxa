use serde_json::{Value, json};

pub(crate) struct Provider {
    pub id: &'static str,
    pub token_field: &'static str,
    pub refresh_fields: Option<(&'static str, &'static str)>,
    pub capabilities: &'static [&'static str],
    pub calendar_plan: Option<fn(&str) -> Option<String>>,
    pub snapshot: Option<fn(&Value, &Value) -> Value>,
    pub toggle_watchlist: Option<fn(&Value, &Value, &str, bool) -> Option<Value>>,
    pub mark_watched: Option<fn(&Value, &super::WatchedChange) -> Option<Value>>,
    pub auth_request: Option<fn(&Value, &str) -> Option<Value>>,
    pub token_state: Option<fn(u16, &Value) -> &'static str>,
    pub authorize_url: Option<fn(&Value) -> Option<String>>,
    pub library_requests: Option<fn(&Value) -> Option<Vec<Value>>>,
    pub headers: Option<fn(&str) -> Vec<(&'static str, String)>>,
    pub prepare_url: Option<fn(String, &Value) -> String>,
    pub scrobble: Option<fn(&Value, &str, f64) -> Option<Value>>,
    pub write: Option<fn(&Value, &Value) -> Option<Vec<Value>>>,
    pub verification_key: &'static str,
    pub alt_token_field: Option<&'static str>,
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
        calendar_plan: Some(super::trakt::trakt_calendar_plan_json),
        snapshot: Some(super::trakt::trakt_library_snapshot),
        toggle_watchlist: Some(super::trakt::trakt_toggle_watchlist),
        mark_watched: Some(super::trakt::trakt_mark_watched),
        auth_request: Some(super::trakt::trakt_auth_request),
        token_state: Some(super::trakt::trakt_token_state),
        authorize_url: None,
        library_requests: Some(super::trakt::trakt_library_requests),
        headers: Some(super::trakt::trakt_headers),
        prepare_url: None,
        scrobble: Some(super::trakt::trakt_scrobble),
        write: Some(super::trakt::trakt_write),
        verification_key: "verification_url",
        alt_token_field: None,
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
        snapshot: Some(super::simkl::simkl_library_snapshot),
        toggle_watchlist: Some(super::simkl::simkl_toggle_watchlist),
        mark_watched: Some(super::simkl::simkl_mark_watched),
        auth_request: Some(super::simkl::simkl_auth_request),
        token_state: Some(super::simkl::simkl_token_state),
        authorize_url: Some(super::simkl::simkl_authorize_url),
        library_requests: None,
        headers: Some(super::simkl::simkl_headers),
        prepare_url: Some(super::simkl::simkl_prepare_url),
        scrobble: Some(super::simkl::simkl_scrobble),
        write: Some(super::simkl::simkl_write),
        verification_key: "verification_uri",
        alt_token_field: None,
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
        calendar_plan: Some(super::mdblist::mdblist_calendar_plan_json),
        snapshot: Some(super::mdblist::mdblist_library_snapshot),
        toggle_watchlist: Some(super::mdblist::mdblist_toggle_watchlist),
        mark_watched: Some(super::mdblist::mdblist_mark_watched),
        auth_request: Some(super::mdblist::mdblist_auth_request),
        token_state: Some(super::mdblist::mdblist_token_state),
        authorize_url: None,
        library_requests: Some(super::mdblist::mdblist_library_requests),
        headers: Some(super::mdblist::mdblist_headers),
        prepare_url: Some(super::mdblist::mdblist_prepare_url),
        scrobble: Some(super::mdblist::mdblist_scrobble),
        write: None,
        verification_key: "verification_uri",
        alt_token_field: Some("mdblistApiKey"),
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
        calendar_plan: Some(super::anilist::anilist_calendar_plan_json),
        snapshot: Some(super::anilist::anilist_library_snapshot),
        toggle_watchlist: Some(super::anilist::anilist_toggle_watchlist),
        mark_watched: Some(super::anilist::anilist_mark_watched),
        auth_request: Some(super::anilist::anilist_auth_request),
        token_state: Some(super::anilist::anilist_token_state),
        authorize_url: Some(super::anilist::anilist_authorize_url),
        library_requests: Some(super::anilist::anilist_library_requests),
        headers: Some(super::anilist::anilist_headers),
        prepare_url: None,
        scrobble: None,
        write: None,
        verification_key: "verification_uri",
        alt_token_field: None,
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
        set(self.token_field) || self.alt_token_field.is_some_and(set)
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
