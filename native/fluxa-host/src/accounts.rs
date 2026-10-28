use std::time::Duration;

use web_time::Instant;

use fluxa_ui::AccountPrompt;
use serde_json::{Value, json};

use crate::{RendererState, core_value, host_log, profiles};

pub(crate) struct AccountAuth {
    provider: String,
    device: Option<Value>,
    generation: u64,
    next_poll: Option<Instant>,
    redirect: Option<Redirect>,
}

struct Redirect {
    verifier: String,
    state: String,
    returned: bool,
}

fn auth_state(state: &RendererState) -> Value {
    state
        .core_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("auth"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn generation(auth: &Value) -> u64 {
    auth.get("generation").and_then(Value::as_u64).unwrap_or(0)
}

fn dispatch(state: &RendererState, command: Value) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}

fn has_capability(provider: &str, capability: &str) -> bool {
    core_value("providerRegistry", json!({}))
        .and_then(|registry| registry.as_array().cloned())
        .into_iter()
        .flatten()
        .find(|entry| entry["id"] == provider)
        .is_some_and(|entry| {
            entry["capabilities"]
                .as_array()
                .is_some_and(|all| all.iter().any(|item| item == capability))
        })
}

pub(crate) fn toggle(state: &mut RendererState, provider: &str) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let pending_link = state
        .account_auth
        .as_ref()
        .filter(|flow| flow.provider == provider)
        .and_then(|flow| flow.device.as_ref())
        .and_then(|device| {
            ["verificationUrlComplete", "verificationUrl"]
                .iter()
                .filter_map(|key| device.get(*key).and_then(Value::as_str))
                .find(|url| url.starts_with("http"))
        })
        .map(str::to_owned);
    if let Some(link) = pending_link {
        state.open_url = Some(link);
        return;
    }
    let profile = session.active_profile();
    let token = format!("{provider}AccessToken");
    let connected = profile
        .get(&token)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
        && !state
            .settings
            .tracker_names
            .iter()
            .any(|(name, handle)| name == provider && handle.is_empty());
    if !connected {
        state.settings.account_auth = None;
        let generation = generation(&auth_state(state));
        let browser = has_capability(provider, "browser")
            && state.home.form_factor != fluxa_ui::UiFormFactorJson::Tv
            && matches!(
                state.home.platform,
                fluxa_ui::UiPlatform::Android
                    | fluxa_ui::UiPlatform::Ios
                    | fluxa_ui::UiPlatform::Linux
                    | fluxa_ui::UiPlatform::Windows
                    | fluxa_ui::UiPlatform::Macos
            );
        dispatch(
            state,
            json!({
                "type": "authFlowRequested",
                "provider": provider,
                "mode": if browser { "pkce" } else { "device" },
            }),
        );
        state.account_auth = Some(AccountAuth {
            provider: provider.to_owned(),
            device: None,
            generation,
            next_poll: None,
            redirect: None,
        });
        return;
    }
    disconnect(state, provider);
}

fn profile_id(session: &fluxa_effects::SessionHandle) -> String {
    session
        .active_profile()
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("guest")
        .to_owned()
}

#[derive(Default)]
pub(crate) struct MdblistLists {
    key: String,
    lists: Vec<(String, String)>,
    rx: Option<std::sync::mpsc::Receiver<Vec<Value>>>,
}

pub(crate) fn poll_mdblist_lists(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let profile = session.active_profile();
    let token = profile
        .get("mdblistAccessToken")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let api_key = state
        .settings
        .str_value("mdblistApiKey")
        .unwrap_or_default();
    let key = if token.is_empty() && api_key.is_empty() {
        String::new()
    } else {
        format!("{}|{token}|{api_key}", profile_id(session))
    };
    let fetch = &mut state.mdblist_lists;
    if fetch.key != key {
        fetch.key = key.clone();
        fetch.lists.clear();
        fetch.rx = (!key.is_empty()).then(|| session.executor().fetch_mdblist_lists(profile));
    }
    if let Some(lists) = fetch.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        fetch.rx = None;
        fetch.lists = lists
            .iter()
            .filter_map(|list| {
                Some((
                    list.get("id")?.as_str()?.to_owned(),
                    list.get("name")?.as_str()?.to_owned(),
                ))
            })
            .collect();
    }
    if state.settings.mdblist_lists != state.mdblist_lists.lists {
        state.settings.mdblist_lists = state.mdblist_lists.lists.clone();
    }
}

#[derive(Default)]
pub(crate) struct TrackerProfiles {
    key: String,
    names: Vec<(String, String)>,
    rx: Option<std::sync::mpsc::Receiver<Vec<Value>>>,
}

pub(crate) fn poll_tracker_profiles(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let profile = session.active_profile();
    let tokens = [
        "traktAccessToken",
        "simklAccessToken",
        "anilistAccessToken",
        "mdblistAccessToken",
    ]
    .iter()
    .map(|key| {
        profile
            .get(*key)
            .and_then(Value::as_str)
            .unwrap_or_default()
    })
    .collect::<Vec<_>>();
    let key = if tokens.iter().all(|token| token.is_empty()) {
        String::new()
    } else {
        format!("{}|{}", profile_id(session), tokens.join("|"))
    };
    let fetch = &mut state.tracker_profiles;
    if fetch.key != key {
        let same_profile = key.split('|').next() == fetch.key.split('|').next();
        fetch.key = key.clone();
        if !same_profile {
            fetch.names.clear();
        }
        fetch.rx = (!key.is_empty()).then(|| session.executor().fetch_tracker_profiles(profile));
    }
    if let Some(found) = fetch.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        fetch.rx = None;
        fetch.names = found
            .iter()
            .filter_map(|entry| {
                Some((
                    entry.get("provider")?.as_str()?.to_owned(),
                    entry.get("name")?.as_str()?.to_owned(),
                ))
            })
            .collect();
    }
    if state.settings.tracker_names != state.tracker_profiles.names {
        state.settings.tracker_names = state.tracker_profiles.names.clone();
    }
}

pub(crate) fn refresh_servers(state: &mut RendererState) {
    state.media_servers_profile = state.session.as_ref().map(profile_id);
    state.settings.media_servers = state
        .session
        .as_ref()
        .map(|session| fluxa_effects::media_servers(session.storage(), &profile_id(session)))
        .unwrap_or_default();
}

pub(crate) fn refresh_servers_if_stale(state: &mut RendererState) {
    let current = state.session.as_ref().map(profile_id);
    if state.route == crate::Route::Settings || current != state.media_servers_profile {
        refresh_servers(state);
    }
}

fn start_flow(state: &mut RendererState, provider: &str, command: Value) {
    state.settings.account_auth = None;
    let generation = generation(&auth_state(state));
    dispatch(state, command);
    state.account_auth = Some(AccountAuth {
        provider: provider.to_owned(),
        device: None,
        generation,
        next_poll: None,
        redirect: None,
    });
}

pub(crate) fn media_server(state: &mut RendererState, node: u64) {
    let kinds = [
        fluxa_ui::NODE_SETTINGS_SERVER_JELLYFIN,
        fluxa_ui::NODE_SETTINGS_SERVER_EMBY,
        fluxa_ui::NODE_SETTINGS_SERVER_PLEX,
    ];
    if let Some(kind) = kinds.iter().position(|kind| *kind == node) {
        state.settings.server_kind = kind;
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    if let Some(index) = fluxa_ui::server_index(node) {
        let key = state
            .settings
            .media_servers
            .get(index)
            .and_then(|server| server.get("key"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        if let Some(key) = key
            && let Err(error) =
                fluxa_effects::remove_media_server(session.storage(), &profile_id(session), &key)
        {
            host_log(format!("Media server removal failed: {error}"));
        }
        refresh_servers(state);
        reload_addons(state);
        return;
    }
    if state.settings.server_kind == 2 {
        start_flow(
            state,
            "plex",
            json!({"type": "authFlowRequested", "provider": "plex", "mode": "device"}),
        );
        return;
    }
    let provider = if state.settings.server_kind == 1 {
        "emby"
    } else {
        "jellyfin"
    };
    let [address, username, password] = state.settings.server_fields.clone();
    if address.trim().is_empty() {
        return;
    }
    let credentials = json!({
        "baseUrl": address.trim(),
        "username": username.trim(),
        "password": password,
    });
    let command = json!({
        "type": "authExchangeRequested",
        "provider": provider,
        "code": credentials.to_string(),
        "codeVerifier": "",
        "profile": session.active_profile(),
    });
    start_flow(state, provider, command);
}

fn reload_addons(state: &RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    dispatch(
        state,
        json!({
            "type": "addonsRefreshRequested",
            "profile": session.active_profile(),
            "forceRefresh": true,
        }),
    );
}

pub(crate) fn disconnect(state: &mut RendererState, provider: &str) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let mut profile = session.active_profile();
    if let Some(fields) = profile.as_object_mut() {
        fields.retain(|key, _| !(key.starts_with(provider) && key.contains("Token")));
    }
    profiles::save_profile(session.storage(), &profile);
    dispatch(
        state,
        json!({"type": "profileActivated", "profile": profile.clone()}),
    );
    if let Some(session) = state.session.as_ref()
        && let Err(error) = profiles::load_profile_with(session, profile, true)
    {
        host_log(format!("Profile load failed: {error}"));
    }
    if state
        .account_auth
        .as_ref()
        .is_some_and(|flow| flow.provider == provider)
    {
        state.account_auth = None;
    }
}

pub(crate) fn finish_redirect(state: &mut RendererState, url: &str) {
    let Some(callback) = core_value("providerAuthCallback", json!({"url": url})) else {
        return;
    };
    let provider = callback
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(flow) = state.account_auth.as_mut() else {
        return;
    };
    let Some(redirect) = flow.redirect.as_mut() else {
        return;
    };
    let code = callback
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let matches_state = callback.get("state").and_then(Value::as_str) == Some(&redirect.state);
    if flow.provider != provider || redirect.returned || code.is_empty() || !matches_state {
        if flow.provider == provider && !redirect.returned {
            state.settings.account_auth = Some(AccountPrompt {
                provider: provider.to_owned(),
                code: String::new(),
                url: String::new(),
                failed: true,
            });
            state.account_auth = None;
        }
        return;
    }
    redirect.returned = true;
    let provider = flow.provider.clone();
    let command = json!({
        "type": "authExchangeRequested",
        "provider": provider,
        "code": code,
        "codeVerifier": redirect.verifier,
        "profile": state.session.as_ref().map(|session| session.active_profile()),
    });
    dispatch(state, command);
}

pub(crate) fn poll(state: &mut RendererState) {
    let auth = auth_state(state);
    let Some(flow) = state.account_auth.as_mut() else {
        if state
            .settings
            .account_auth
            .as_ref()
            .is_some_and(|prompt| !prompt.failed)
        {
            state.settings.account_auth = None;
        }
        return;
    };
    let now = Instant::now();
    let settled = generation(&auth) > flow.generation
        && !auth
            .get("isLoading")
            .and_then(Value::as_bool)
            .unwrap_or(true);
    if settled {
        flow.generation = generation(&auth);
        let result = auth.get("result").cloned().unwrap_or(Value::Null);
        let interval = flow
            .device
            .as_ref()
            .or(result.get("device"))
            .and_then(|device| device.get("interval"))
            .and_then(Value::as_u64)
            .unwrap_or(5);
        match result.get("state").and_then(Value::as_str) {
            Some("success") => {
                let provider = flow.provider.clone();
                state.account_auth = None;
                state.settings.account_auth = None;
                if matches!(provider.as_str(), "plex" | "jellyfin" | "emby") {
                    state.settings.server_fields = Default::default();
                    refresh_servers(state);
                    reload_addons(state);
                }
                if let Some(session) = state.session.as_ref() {
                    if let Some(profile) = result.get("profile") {
                        profiles::save_profile(session.storage(), profile);
                    }
                    if let Err(error) = profiles::load_profile(session, true) {
                        host_log(format!("Profile load failed: {error}"));
                    }
                }
                return;
            }
            Some("redirect") => {
                let field = |key: &str| {
                    result
                        .get(key)
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned()
                };
                flow.redirect = Some(Redirect {
                    verifier: field("codeVerifier"),
                    state: field("oauthState"),
                    returned: false,
                });
                state.open_url = Some(field("url"));
                return;
            }
            Some("pending") => {
                if flow.device.is_none() {
                    flow.device = result.get("device").cloned();
                    let link = flow.device.as_ref().and_then(|device| {
                        ["verificationUrlComplete", "verificationUrl"]
                            .iter()
                            .filter_map(|key| device.get(*key).and_then(Value::as_str))
                            .find(|url| url.starts_with("http"))
                    });
                    if let Some(link) = link
                        && state.home.form_factor != fluxa_ui::UiFormFactorJson::Tv
                    {
                        state.open_url = Some(link.to_owned());
                    }
                }
                flow.next_poll = Some(now + Duration::from_secs(interval));
            }
            Some("slow_down") => flow.next_poll = Some(now + Duration::from_secs(interval + 5)),
            _ => {
                if let Some(error) = auth.get("error").filter(|error| !error.is_null()) {
                    host_log(format!("{} sign-in failed: {error}", flow.provider));
                }
                state.settings.account_auth = Some(AccountPrompt {
                    provider: flow.provider.clone(),
                    code: String::new(),
                    url: String::new(),
                    failed: true,
                });
                state.account_auth = None;
                return;
            }
        }
    }
    let expired = flow
        .device
        .as_ref()
        .and_then(|device| device.get("expiresAt"))
        .and_then(Value::as_i64)
        .is_some_and(|expires| expires < chrono::Utc::now().timestamp());
    if expired {
        state.account_auth = None;
        state.settings.account_auth = None;
        return;
    }
    let field = |key: &str| {
        flow.device
            .as_ref()
            .and_then(|device| device.get(key))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    state.settings.account_auth = flow.device.as_ref().map(|_| AccountPrompt {
        provider: flow.provider.clone(),
        code: field("userCode"),
        url: field("verificationUrl"),
        failed: false,
    });
    let Some(due) = flow.next_poll else {
        return;
    };
    if due > now {
        state.redraw_at = Some(state.redraw_at.map_or(due, |at| at.min(due)));
        return;
    }
    flow.next_poll = None;
    let command = json!({
        "type": "authExchangeRequested",
        "provider": flow.provider,
        "code": field("deviceCode"),
        "codeVerifier": "",
        "profile": state.session.as_ref().map(|session| session.active_profile()),
    });
    dispatch(state, command);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_account_provider_can_sign_in() {
        for provider in fluxa_ui::ACCOUNT_PROVIDERS {
            assert!(
                has_capability(provider, "device") || has_capability(provider, "browser"),
                "{provider} has no sign-in flow in the registry"
            );
        }
    }
}
