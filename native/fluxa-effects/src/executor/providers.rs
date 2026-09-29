use super::{EffectExecutor, NativeTimeout, chrono_unix_seconds, core_value};
use crate::storage::{Storage, sanitize_key};
use reqwest::{Client, Method};
use serde_json::{Map, Value, json};
use std::time::Duration;

pub(super) fn is_provider(source: &str) -> bool {
    matches!(source, "trakt" | "simkl" | "mdblist")
}

fn client_id(provider: &str) -> &'static str {
    match provider {
        "trakt" => option_env!("FLUXA_TRAKT_CLIENT_ID").unwrap_or(""),
        "simkl" => option_env!("FLUXA_SIMKL_CLIENT_ID").unwrap_or(""),
        _ => "",
    }
}

fn client_secret(provider: &str) -> &'static str {
    match provider {
        "trakt" => option_env!("FLUXA_TRAKT_CLIENT_SECRET").unwrap_or(""),
        "simkl" => option_env!("FLUXA_SIMKL_CLIENT_SECRET").unwrap_or(""),
        _ => "",
    }
}

fn token_field(provider: &str) -> &'static str {
    match provider {
        "trakt" => "traktAccessToken",
        "simkl" => "simklAccessToken",
        _ => "",
    }
}

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .native_timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())
}

async fn send(client: &Client, plan: &Value) -> Result<(u16, Value), String> {
    let method = Method::from_bytes(str_field(plan, "method").as_bytes())
        .map_err(|error| error.to_string())?;
    let mut request = client.request(method, str_field(plan, "url"));
    for (name, value) in plan
        .get("headers")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if let Some(value) = value.as_str().filter(|value| !value.is_empty()) {
            request = request.header(name, value);
        }
    }
    if let Some(body) = plan.get("body").filter(|body| !body.is_null()) {
        request = request.body(body.to_string());
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
}

impl EffectExecutor {
    fn provider_api_key(&self, profile: &Value) -> String {
        let profile_id = str_field(profile, "id");
        [
            Some(profile.clone()),
            self.storage
                .read_json(&Storage::prefs_key(profile_id))
                .ok()
                .flatten(),
            self.storage.read_json("prefs").ok().flatten(),
        ]
        .into_iter()
        .flatten()
        .find_map(|source| {
            source
                .get("mdblistApiKey")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default()
    }

    fn provider_credentials(&self, provider: &str, profile: &Value) -> Option<Value> {
        let token = str_field(profile, token_field(provider));
        let api_key = self.provider_api_key(profile);
        let connected = match provider {
            "mdblist" => !api_key.is_empty(),
            _ => !token.is_empty() && !client_id(provider).is_empty(),
        };
        connected.then(|| {
            json!({
                "provider": provider,
                "token": token,
                "clientId": client_id(provider),
                "apiKey": api_key,
            })
        })
    }

    fn save_profile(&self, profile: &Value) {
        let profiles = self
            .storage
            .read_json("profiles")
            .ok()
            .flatten()
            .unwrap_or_else(|| json!([]));
        if let Some(next) = core_value(
            "profileMutationPlan",
            json!({"operation": "save", "profiles": profiles, "profile": profile}),
        ) {
            let _ = self.storage.write_json("profiles", &next);
        }
    }

    fn merge_auth(&self, provider: &str, profile: &Value, auth: &Value) -> Result<Value, String> {
        let merged = core_value(
            "tokenMergePlan",
            json!({"provider": provider, "profile": profile, "authResult": auth}),
        )
        .and_then(|plan| plan.get("mergedProfile").cloned())
        .ok_or_else(|| "Fluxa Core could not merge the provider token".to_owned())?;
        self.save_profile(&merged);
        Ok(merged)
    }

    async fn auth_call(
        &self,
        provider: &str,
        operation: &str,
        extra: Value,
    ) -> Result<Value, String> {
        let mut args = json!({
            "provider": provider,
            "operation": operation,
            "clientId": client_id(provider),
            "clientSecret": client_secret(provider),
        });
        if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
            args.extend(extra.clone());
        }
        let plan = core_value("providerAuthRequest", args)
            .ok_or_else(|| format!("{provider} sign-in is not available"))?;
        let (status, body) = send(&http_client()?, &plan).await?;
        core_value(
            "providerAuthOutcome",
            json!({
                "provider": provider,
                "operation": operation,
                "status": status,
                "body": body,
                "nowSeconds": chrono_unix_seconds(),
            }),
        )
        .ok_or_else(|| "Fluxa Core could not read the sign-in response".to_owned())
    }

    pub(super) async fn run_auth_flow(&self, payload: &Value) -> Result<Value, String> {
        let provider = str_field(payload, "provider");
        if client_id(provider).is_empty() {
            return Err(format!("{provider} is not configured in this build"));
        }
        let outcome = self.auth_call(provider, "start", json!({})).await?;
        if str_field(&outcome, "state") != "pending" {
            return Err(format!("{provider} did not return a device code"));
        }
        Ok(json!({"provider": provider, "state": "pending", "device": outcome.get("device")}))
    }

    pub(super) async fn exchange_auth_code(&self, payload: &Value) -> Result<Value, String> {
        let provider = str_field(payload, "provider");
        let outcome = self
            .auth_call(
                provider,
                "poll",
                json!({"code": str_field(payload, "code")}),
            )
            .await?;
        match str_field(&outcome, "state") {
            "success" => {
                let profile = self.merge_auth(
                    provider,
                    payload.get("profile").unwrap_or(&Value::Null),
                    outcome.get("auth").unwrap_or(&Value::Null),
                )?;
                Ok(json!({"provider": provider, "state": "success", "profile": profile}))
            }
            state @ ("pending" | "slow_down") => Ok(json!({"provider": provider, "state": state})),
            state => Err(format!("{provider} sign-in failed: {state}")),
        }
    }

    pub(super) async fn refresh_auth_token(&self, payload: &Value) -> Result<Value, String> {
        let provider = str_field(payload, "provider");
        let profile = payload.get("profile").cloned().unwrap_or(Value::Null);
        let profile = self.refreshed_profile(provider, profile, true).await?;
        Ok(json!({"provider": provider, "profile": profile}))
    }

    async fn refreshed_profile(
        &self,
        provider: &str,
        profile: Value,
        force: bool,
    ) -> Result<Value, String> {
        let refresh_token = str_field(&profile, "traktRefreshToken");
        let expires_at = profile
            .get("traktTokenExpiresAt")
            .and_then(Value::as_i64)
            .unwrap_or(i64::MAX);
        let due = force || expires_at < chrono_unix_seconds() + 86_400;
        if provider != "trakt" || refresh_token.is_empty() || !due {
            return Ok(profile);
        }
        let outcome = self
            .auth_call(provider, "refresh", json!({"refreshToken": refresh_token}))
            .await?;
        if str_field(&outcome, "state") != "success" {
            return Err("Trakt session expired".to_owned());
        }
        self.merge_auth(
            provider,
            &profile,
            outcome.get("auth").unwrap_or(&Value::Null),
        )
    }

    fn provider_cache_key(provider: &str, profile_id: &str) -> String {
        format!("provider_library_{provider}_{}", sanitize_key(profile_id))
    }

    pub(super) fn cached_provider_library(
        &self,
        provider: &str,
        profile_id: &str,
    ) -> Option<Value> {
        self.storage
            .read_json(&Self::provider_cache_key(provider, profile_id))
            .ok()
            .flatten()
    }

    pub(super) fn store_provider_library(&self, provider: &str, profile_id: &str, library: &Value) {
        let _ = self
            .storage
            .write_json(&Self::provider_cache_key(provider, profile_id), library);
    }

    pub(super) async fn read_provider_library(
        &self,
        provider: &str,
        profile_id: &str,
        profile: &Value,
    ) -> Result<Value, String> {
        let profile = self
            .refreshed_profile(provider, profile.clone(), false)
            .await?;
        let credentials = self
            .provider_credentials(provider, &profile)
            .ok_or_else(|| format!("{provider} is not connected to the active profile"))?;
        let requests = core_value("providerLibraryRequests", credentials)
            .and_then(|requests| requests.as_array().cloned())
            .ok_or_else(|| "Fluxa Core could not plan the library requests".to_owned())?;
        let client = http_client()?;
        let results = futures::future::join_all(requests.iter().map(|request| {
            let client = &client;
            async move {
                (
                    str_field(request, "key").to_owned(),
                    send(client, request).await,
                )
            }
        }))
        .await;
        let mut responses = Map::new();
        let mut failures = Vec::new();
        for (key, result) in results {
            match result {
                Ok((status, body)) if (200..300).contains(&status) => {
                    responses.insert(key, body);
                }
                Ok((status, _)) => failures.push(format!("{key}: {status}")),
                Err(error) => failures.push(format!("{key}: {error}")),
            }
        }
        if !failures.is_empty() {
            crate::log!("[fluxa-native] {provider} library requests failed: {failures:?}");
        }
        if responses.is_empty() {
            return self
                .cached_provider_library(provider, profile_id)
                .ok_or_else(|| format!("{provider} library could not be loaded"));
        }
        let snapshot = core_value(
            "providerLibrarySnapshot",
            json!({"provider": provider, "responses": responses}),
        )
        .ok_or_else(|| "Fluxa Core could not build the provider library".to_owned())?;
        self.store_provider_library(provider, profile_id, &snapshot);
        Ok(snapshot)
    }

    pub(super) async fn push_provider_command(
        &self,
        provider: &str,
        profile: &Value,
        command: &Value,
    ) -> Result<(), String> {
        let Some(mut args) = self.provider_credentials(provider, profile) else {
            return Err(format!("{provider} is not connected to the active profile"));
        };
        args["command"] = command.clone();
        let Some(requests) = core_value("providerWriteRequests", args) else {
            return Ok(());
        };
        let client = http_client()?;
        for request in requests.as_array().into_iter().flatten() {
            let (status, body) = send(&client, request).await?;
            if !(200..300).contains(&status) {
                return Err(format!("{provider} rejected the change ({status}): {body}"));
            }
        }
        Ok(())
    }

    pub(super) async fn sync_watched_state(&self, payload: &Value) -> Result<Value, String> {
        let profile = payload.get("profile").cloned().unwrap_or(Value::Null);
        let library_source = str_field(&profile, "integrationLibrarySource");
        let meta = payload.get("meta").cloned().unwrap_or(Value::Null);
        let meta_id = str_field(&meta, "id");
        let video_ids: Vec<Value> = payload
            .get("episodes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|episode| episode.get("id").cloned())
            .collect();
        let command = json!({
            "type": "markWatched",
            "seriesId": meta_id,
            "videoIds": if video_ids.is_empty() { json!([meta_id]) } else { json!(video_ids) },
            "watched": payload.get("watched").and_then(Value::as_bool).unwrap_or(true),
        });
        for provider in ["trakt", "simkl", "mdblist"] {
            if provider == library_source || self.provider_credentials(provider, &profile).is_none()
            {
                continue;
            }
            if let Err(error) = self
                .push_provider_command(provider, &profile, &command)
                .await
            {
                crate::log!("[fluxa-native] {provider} watched sync failed: {error}");
            }
        }
        Ok(json!({}))
    }

    pub(super) async fn scrobble(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = self.active_profile_id(payload.get("profile"))?;
        let profile = payload
            .get("profile")
            .filter(|profile| profile.is_object())
            .cloned()
            .or_else(|| {
                self.storage
                    .read_json("profiles")
                    .ok()
                    .flatten()?
                    .as_array()?
                    .iter()
                    .find(|profile| str_field(profile, "id") == profile_id)
                    .cloned()
            })
            .unwrap_or(Value::Null);
        let client = http_client()?;
        for provider in ["trakt", "simkl", "mdblist"] {
            let Some(mut args) = self.provider_credentials(provider, &profile) else {
                continue;
            };
            args["action"] = payload.get("actionName").cloned().unwrap_or(Value::Null);
            args["itemId"] = payload.get("itemId").cloned().unwrap_or(Value::Null);
            args["metaType"] = payload.get("metaType").cloned().unwrap_or(Value::Null);
            args["progress"] = payload.get("progress").cloned().unwrap_or(Value::Null);
            let Some(plan) = core_value("providerScrobbleRequest", args) else {
                continue;
            };
            match send(&client, &plan).await {
                Ok((status, _)) if (200..300).contains(&status) || status == 409 => {}
                Ok((status, body)) => {
                    crate::log!("[fluxa-native] {provider} scrobble rejected ({status}): {body}")
                }
                Err(error) => crate::log!("[fluxa-native] {provider} scrobble failed: {error}"),
            }
        }
        Ok(json!({}))
    }
}
