use super::helpers::normalize_error;
use super::profile;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct AuthState {
    provider: String,
    mode: String,
    is_loading: bool,
    result: Value,
    error: Value,
    generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunAuthFlowPayload {
    provider: String,
    mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExchangeAuthCodePayload {
    provider: String,
    code: String,
    code_verifier: Option<String>,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshAuthTokenPayload {
    provider: String,
    profile: Value,
}

pub(super) fn dispatch_flow(
    engine: &mut HeadlessEngine,
    provider: String,
    mode: String,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Auth);
    *engine.state.auth = AuthState {
        provider: provider.clone(),
        mode: mode.clone(),
        is_loading: true,
        result: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::RunAuthFlow,
        generation,
        RunAuthFlowPayload { provider, mode },
    )]
}

pub(super) fn dispatch_exchange(
    engine: &mut HeadlessEngine,
    provider: String,
    code: String,
    code_verifier: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Auth);
    *engine.state.auth = AuthState {
        provider: provider.clone(),
        mode: "exchange".to_string(),
        is_loading: true,
        result: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::ExchangeAuthCode,
        generation,
        ExchangeAuthCodePayload {
            provider,
            code,
            code_verifier,
            profile: profile.unwrap_or_else(|| engine.state.profile.active.clone()),
        },
    )]
}

pub(super) fn dispatch_token_refresh(
    engine: &mut HeadlessEngine,
    provider: String,
    profile: Value,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Auth);
    *engine.state.auth = AuthState {
        provider: provider.clone(),
        mode: "refresh".to_string(),
        is_loading: true,
        result: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::RefreshAuthToken,
        generation,
        RefreshAuthTokenPayload { provider, profile },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "runAuthFlow" => {
            if generation == engine.state.runtime.get(GenerationKey::Auth) {
                engine.state.auth.is_loading = false;
                if result.status.is_ok() {
                    engine.state.auth.result = result.value.clone();
                    engine.state.auth.error = Value::Null;
                } else {
                    engine.state.auth.error = normalize_error(result.error.clone());
                }
            }
        }
        "exchangeAuthCode" | "refreshAuthToken" => {
            if generation == engine.state.runtime.get(GenerationKey::Auth) {
                engine.state.auth.is_loading = false;
                if result.status.is_ok() {
                    engine.state.auth.result = result.value.clone();
                    if let Some(updated_profile) = result.value.get("profile").cloned() {
                        profile::update_active(engine, updated_profile);
                    }
                    engine.state.auth.error = Value::Null;
                } else {
                    engine.state.auth.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
