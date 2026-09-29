use std::time::Duration;

use web_time::Instant;

use fluxa_ui::AccountPrompt;
use serde_json::{Value, json};

use crate::{RendererState, card_menu, core_value, host_log, profiles};

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

pub(crate) fn toggle(state: &mut RendererState, provider: &str) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let mut profile = session.active_profile();
    let token = format!("{provider}AccessToken");
    let connected = profile
        .get(&token)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty());
    if !connected {
        state.settings.account_auth = None;
        let generation = generation(&auth_state(state));
        let browser = matches!(provider, "simkl" | "anilist")
            && state.home.form_factor == fluxa_ui::UiFormFactorJson::Mobile
            && matches!(
                state.home.platform,
                fluxa_ui::UiPlatform::Android | fluxa_ui::UiPlatform::Ios
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
    card_menu::open_disconnect(state, provider);
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
        json!({"type": "profileActivated", "profile": profile}),
    );
    if let Some(session) = state.session.as_ref()
        && let Err(error) = profiles::load_profile(session)
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
                state.account_auth = None;
                state.settings.account_auth = None;
                if let Some(session) = state.session.as_ref() {
                    if let Some(profile) = result.get("profile") {
                        profiles::save_profile(session.storage(), profile);
                    }
                    if let Err(error) = profiles::load_profile(session) {
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
