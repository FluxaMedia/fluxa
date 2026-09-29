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
        "mdblist" => option_env!("FLUXA_MDBLIST_CLIENT_ID").unwrap_or(""),
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
        "mdblist" => "mdblistAccessToken",
        _ => "",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn random_hex() -> Option<String> {
    use aes_gcm::aead::Generate;
    use aes_gcm::{Aes256Gcm, Key};
    let key = Key::<Aes256Gcm>::generate();
    Some(key.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(target_arch = "wasm32")]
fn random_hex() -> Option<String> {
    None
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

const TRAKT_SEASONS_TTL: i64 = 24 * 60 * 60;
const SIMKL_ACCOUNT_TTL: i64 = 86_400;

fn find_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => map
            .get(key)
            .or_else(|| map.values().find_map(|inner| find_key(inner, key))),
        Value::Array(items) => items.iter().find_map(|inner| find_key(inner, key)),
        _ => None,
    }
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
        let form = plan["headers"]["Content-Type"]
            .as_str()
            .is_some_and(|kind| kind.contains("x-www-form-urlencoded"));
        request = match body.as_object().filter(|_| form) {
            Some(fields) => request.form(
                &fields
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str().unwrap_or_default()))
                    .collect::<Vec<_>>(),
            ),
            None => request.body(body.to_string()),
        };
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
            "mdblist" => !token.is_empty() || !api_key.is_empty(),
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
        if str_field(payload, "mode") == "pkce" {
            let verifier = random_hex().ok_or("secure randomness is unavailable")?;
            let oauth_state = random_hex().ok_or("secure randomness is unavailable")?;
            let authorize = core_value(
                "providerAuthorizeUrl",
                json!({
                    "provider": provider,
                    "clientId": client_id(provider),
                    "codeVerifier": verifier,
                    "state": oauth_state,
                }),
            )
            .ok_or_else(|| format!("{provider} does not support browser sign-in"))?;
            return Ok(json!({
                "provider": provider,
                "state": "redirect",
                "url": authorize.get("url"),
                "codeVerifier": verifier,
                "oauthState": oauth_state,
            }));
        }
        let outcome = self.auth_call(provider, "start", json!({})).await?;
        if str_field(&outcome, "state") != "pending" {
            return Err(format!("{provider} did not return a device code"));
        }
        Ok(json!({"provider": provider, "state": "pending", "device": outcome.get("device")}))
    }

    pub(super) async fn exchange_auth_code(&self, payload: &Value) -> Result<Value, String> {
        let provider = str_field(payload, "provider");
        let verifier = str_field(payload, "codeVerifier");
        let (operation, extra) = if verifier.is_empty() {
            ("poll", json!({"code": str_field(payload, "code")}))
        } else {
            (
                "exchange",
                json!({"code": str_field(payload, "code"), "codeVerifier": verifier}),
            )
        };
        let outcome = self.auth_call(provider, operation, extra).await?;
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
        let (refresh_field, expires_field) = match provider {
            "trakt" => ("traktRefreshToken", "traktTokenExpiresAt"),
            "simkl" => ("simklRefreshToken", "simklTokenExpiresAt"),
            "mdblist" => ("mdblistRefreshToken", "mdblistTokenExpiresAt"),
            _ => return Ok(profile),
        };
        let refresh_token = str_field(&profile, refresh_field);
        let expires_at = profile
            .get(expires_field)
            .and_then(Value::as_i64)
            .unwrap_or(i64::MAX);
        let due = force || expires_at < chrono_unix_seconds() + 86_400;
        if refresh_token.is_empty() || !due {
            return Ok(profile);
        }
        let outcome = self
            .auth_call(provider, "refresh", json!({"refreshToken": refresh_token}))
            .await?;
        if str_field(&outcome, "state") != "success" {
            return Err(format!("{provider} session expired, sign in again"));
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
        if provider == "simkl" {
            return self.read_simkl_library(profile_id, credentials).await;
        }
        let requests = core_value("providerLibraryRequests", credentials)
            .and_then(|requests| requests.as_array().cloned())
            .ok_or_else(|| "Fluxa Core could not plan the library requests".to_owned())?;
        let responses = if provider == "trakt" {
            match self.read_trakt_library(&requests).await {
                Ok(responses) => responses,
                Err(error) => {
                    crate::log!("[fluxa-native] trakt library sync failed: {error}");
                    return self
                        .cached_provider_library(provider, profile_id)
                        .ok_or(error);
                }
            }
        } else {
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
            responses
        };
        let snapshot = core_value(
            "providerLibrarySnapshot",
            json!({"provider": provider, "responses": responses, "nowSeconds": chrono_unix_seconds()}),
        )
        .ok_or_else(|| "Fluxa Core could not build the provider library".to_owned())?;
        self.store_provider_library(provider, profile_id, &snapshot);
        Ok(snapshot)
    }

    async fn simkl_get(client: &Client, plan: &Value) -> Result<Value, String> {
        Self::provider_get(client, plan, "Simkl").await
    }

    async fn provider_get(client: &Client, plan: &Value, name: &str) -> Result<Value, String> {
        let mut delay = 1;
        for attempt in 0..4 {
            let (status, body) = send(client, plan).await?;
            match status {
                200..=299 => return Ok(body),
                401 => return Err(format!("{name} access was revoked, sign in again")),
                429 | 500 | 502 | 503 | 504 if attempt < 3 => {
                    #[cfg(not(target_arch = "wasm32"))]
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    delay *= 2;
                }
                _ => return Err(format!("{name} request failed ({status})")),
            }
        }
        Err(format!("{name} request failed"))
    }

    async fn read_trakt_library(&self, requests: &[Value]) -> Result<Map<String, Value>, String> {
        let client = http_client()?;
        let mut responses = Map::new();
        for request in requests {
            let key = str_field(request, "key").to_owned();
            let body = if key == "history" {
                Self::trakt_history(&client, request).await?
            } else {
                Self::provider_get(&client, request, "Trakt").await?
            };
            responses.insert(key, body);
        }
        Ok(responses)
    }

    async fn trakt_history(client: &Client, request: &Value) -> Result<Value, String> {
        let mut items = Vec::new();
        for page in 1..=20 {
            let mut request = request.clone();
            let url = str_field(&request, "url").replace("page=1", &format!("page={page}"));
            request["url"] = json!(url);
            let body = Self::provider_get(client, &request, "Trakt").await?;
            let batch = body.as_array().cloned().unwrap_or_default();
            let last = batch.len() < 100;
            items.extend(batch);
            if last {
                break;
            }
        }
        Ok(Value::Array(items))
    }

    async fn simkl_pull(
        &self,
        state_key: &str,
        credentials: &Value,
        state: Option<Value>,
    ) -> Result<Option<Value>, String> {
        let client = http_client()?;
        let plan_for = |activities: Option<&Value>| {
            let mut args = credentials.clone();
            args["state"] = state.clone().unwrap_or(Value::Null);
            args["nowSeconds"] = json!(chrono_unix_seconds());
            if let Some(activities) = activities {
                args["activities"] = activities.clone();
            }
            core_value("simklSyncPlan", args)
                .ok_or_else(|| "Fluxa Core could not plan the Simkl sync".to_owned())
        };
        let first = plan_for(None)?;
        let request = first["requests"][0].clone();
        let activities = Self::simkl_get(&client, &request).await?;
        let plan = plan_for(Some(&activities))?;
        let mode = str_field(&plan, "mode").to_owned();
        if mode == "cached" {
            return Ok(None);
        }
        let mut responses = Map::new();
        for request in plan["requests"].as_array().into_iter().flatten() {
            let body = Self::simkl_get(&client, request).await?;
            responses.insert(str_field(request, "key").to_owned(), body);
        }
        let applied = core_value(
            "simklSyncApply",
            json!({
                "mode": mode,
                "nowSeconds": chrono_unix_seconds(),
                "state": state.unwrap_or(Value::Null),
                "activities": activities,
                "responses": responses,
            }),
        )
        .ok_or_else(|| "Fluxa Core could not build the Simkl library".to_owned())?;
        let _ = self.storage.write_json(state_key, &applied["state"]);
        Ok(applied.get("snapshot").cloned())
    }

    pub(super) async fn simkl_calendar_items(
        &self,
        profile_id: &str,
        profile: &Value,
        year: i64,
        month: i64,
    ) -> Value {
        use chrono::Datelike;
        let Some(credentials) = self.provider_credentials("simkl", profile) else {
            return json!([]);
        };
        let now = chrono::Utc::now();
        let Some(plan) = core_value(
            "simklCalendarPlan",
            json!({
                "clientId": str_field(&credentials, "clientId"),
                "year": year,
                "month": month,
                "nowYear": now.year(),
                "nowMonth": now.month(),
            }),
        ) else {
            return json!([]);
        };
        let cache_key = format!("simkl_calendar_{year}_{month}");
        let cached = self.storage.read_json(&cache_key).ok().flatten();
        let fresh = cached.as_ref().is_some_and(|cached| {
            chrono_unix_seconds() - cached["fetchedAt"].as_i64().unwrap_or(0) < 3 * 60 * 60
        });
        let mut responses = cached
            .as_ref()
            .map(|cached| cached["responses"].clone())
            .filter(|_| fresh)
            .unwrap_or(Value::Null);
        if responses.is_null() && plan.as_array().is_some_and(|plan| !plan.is_empty()) {
            let mut fetched = Map::new();
            if let Ok(client) = http_client() {
                for request in plan.as_array().into_iter().flatten() {
                    match Self::simkl_get(&client, request).await {
                        Ok(body) => {
                            fetched.insert(str_field(request, "key").to_owned(), body);
                        }
                        Err(error) => {
                            crate::log!("[fluxa-native] simkl calendar failed: {error}");
                            break;
                        }
                    }
                }
            }
            if fetched.len() == 3 {
                let _ = self.storage.write_json(
                    &cache_key,
                    &json!({"fetchedAt": chrono_unix_seconds(), "responses": fetched}),
                );
                responses = Value::Object(fetched);
            } else if let Some(cached) = cached {
                responses = cached["responses"].clone();
            }
        }
        let Some(mut args) = responses.as_object().cloned() else {
            return json!([]);
        };
        let library = self
            .cached_provider_library("simkl", profile_id)
            .unwrap_or(Value::Null);
        let allowed: Vec<Value> = ["watchlist", "continueWatching", "onHold"]
            .iter()
            .filter_map(|list| library.get(list).and_then(Value::as_array))
            .flatten()
            .filter_map(|item| item.get("id").cloned())
            .collect();
        args.insert("provider".into(), json!("simkl"));
        args.insert("allowedContentIds".into(), json!(allowed));
        core_value("providerCalendarItems", Value::Object(args)).unwrap_or_else(|| json!([]))
    }

    pub(super) async fn provider_calendar_items(
        &self,
        provider: &str,
        profile: &Value,
        year: i64,
        month: i64,
    ) -> Value {
        let Some(mut credentials) = self.provider_credentials(provider, profile) else {
            return json!([]);
        };
        credentials["year"] = json!(year);
        credentials["month"] = json!(month);
        let Some(plan) = core_value(&format!("{provider}CalendarPlan"), credentials) else {
            return json!([]);
        };
        let cache_key = format!(
            "{provider}_calendar_{}_{year}_{month}",
            sanitize_key(str_field(profile, "id"))
        );
        let cached = self.storage.read_json(&cache_key).ok().flatten();
        let fresh = cached.as_ref().is_some_and(|cached| {
            chrono_unix_seconds() - cached["fetchedAt"].as_i64().unwrap_or(0) < 3 * 60 * 60
        });
        let mut responses = cached
            .as_ref()
            .map(|cached| cached["responses"].clone())
            .filter(|_| fresh);
        if responses.is_none() {
            let mut fetched = Map::new();
            if let Ok(client) = http_client() {
                for request in plan.as_array().into_iter().flatten() {
                    match Self::provider_get(&client, request, provider).await {
                        Ok(body) => {
                            fetched.insert(str_field(request, "key").to_owned(), body);
                        }
                        Err(error) => {
                            crate::log!("[fluxa-native] {provider} calendar failed: {error}");
                            break;
                        }
                    }
                }
            }
            responses = if fetched.len() == plan.as_array().map_or(0, Vec::len) {
                let fetched = Value::Object(fetched);
                let _ = self.storage.write_json(
                    &cache_key,
                    &json!({"fetchedAt": chrono_unix_seconds(), "responses": fetched}),
                );
                Some(fetched)
            } else {
                cached.map(|cached| cached["responses"].clone())
            };
        }
        let Some(responses) = responses else {
            return json!([]);
        };
        core_value(
            "providerCalendarItems",
            json!({
                "provider": provider,
                "shows": responses["shows"],
                "movies": responses["movies"],
                "events": responses["events"]["events"],
            }),
        )
        .unwrap_or_else(|| json!([]))
    }

    async fn read_simkl_library(
        &self,
        profile_id: &str,
        credentials: Value,
    ) -> Result<Value, String> {
        let state_key = format!("simkl_sync_{}", sanitize_key(profile_id));
        let cached = self.cached_provider_library("simkl", profile_id);
        let state = cached
            .as_ref()
            .and_then(|_| self.storage.read_json(&state_key).ok().flatten());
        match self.simkl_pull(&state_key, &credentials, state).await {
            Ok(Some(snapshot)) => {
                self.store_provider_library("simkl", profile_id, &snapshot);
                Ok(snapshot)
            }
            Ok(None) => cached.ok_or_else(|| "Simkl library could not be loaded".to_owned()),
            Err(error) => {
                crate::log!("[fluxa-native] simkl library sync failed: {error}");
                cached.ok_or(error)
            }
        }
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
        let rewatch =
            provider == "simkl" && command.get("rewatch").and_then(Value::as_bool) == Some(true);
        let mut command = command.clone();
        if provider == "trakt" {
            self.remap_trakt_episodes(&args, &mut command).await;
        }
        if rewatch {
            self.ensure_simkl_rewatch(profile, &args).await?;
            command["rewatchId"] = self
                .pinned_rewatch_id(str_field(profile, "id"), str_field(&command, "seriesId"))
                .map_or(Value::Null, |id| json!(id));
        }
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
            if rewatch && let Some(id) = find_key(&body, "rewatch_id").and_then(Value::as_i64) {
                self.pin_rewatch_id(
                    str_field(profile, "id"),
                    str_field(&command, "seriesId"),
                    id,
                );
            }
        }
        Ok(())
    }

    pub(super) async fn trakt_seasons_for_detail(
        &self,
        profile: &Value,
        series_id: &str,
    ) -> Option<Value> {
        if str_field(profile, "integrationLibrarySource") != "trakt" {
            return None;
        }
        let credentials = self.provider_credentials("trakt", profile)?;
        self.trakt_seasons(&credentials, series_id).await
    }

    async fn trakt_seasons(&self, credentials: &Value, series_id: &str) -> Option<Value> {
        let key = format!("trakt_seasons_{}", sanitize_key(series_id));
        let now = chrono_unix_seconds();
        let cached = self.storage.read_json(&key).ok().flatten();
        if let Some(cached) = cached
            .as_ref()
            .filter(|cached| cached["at"].as_i64().unwrap_or(0) + TRAKT_SEASONS_TTL > now)
        {
            return Some(cached["seasons"].clone());
        }
        let mut args = credentials.clone();
        args["command"] = json!({"type": "traktSeasons", "seriesId": series_id});
        let request = core_value("providerWriteRequests", args)?.get(0).cloned()?;
        let fetched = Self::provider_get(&http_client().ok()?, &request, "Trakt").await;
        match fetched {
            Ok(seasons) => {
                let _ = self
                    .storage
                    .write_json(&key, &json!({"at": now, "seasons": seasons}));
                Some(seasons)
            }
            Err(_) => cached.map(|cached| cached["seasons"].clone()),
        }
    }

    async fn remap_trakt_video_ids(
        &self,
        credentials: &Value,
        series_id: &str,
        video_ids: &Value,
        addon_episodes: &Value,
    ) -> Option<Value> {
        if addon_episodes
            .as_array()
            .is_none_or(|episodes| episodes.is_empty())
        {
            return None;
        }
        let seasons = self.trakt_seasons(credentials, series_id).await?;
        core_value(
            "traktRemapVideoIds",
            json!({
                "videoIds": video_ids,
                "addonEpisodes": addon_episodes,
                "traktSeasons": seasons,
            }),
        )
    }

    async fn remap_trakt_episodes(&self, credentials: &Value, command: &mut Value) {
        let addon = command["addonEpisodes"].take();
        if str_field(command, "type") != "markWatched" {
            return;
        }
        let series_id = str_field(command, "seriesId").to_owned();
        if let Some(video_ids) = self
            .remap_trakt_video_ids(credentials, &series_id, &command["videoIds"], &addon)
            .await
        {
            command["videoIds"] = video_ids;
        }
    }

    fn profile_flag(&self, profile: &Value, key: &str) -> bool {
        [
            Some(profile.clone()),
            self.storage
                .read_json(&Storage::prefs_key(str_field(profile, "id")))
                .ok()
                .flatten(),
        ]
        .into_iter()
        .flatten()
        .find_map(|source| source.get(key).and_then(Value::as_bool))
        .unwrap_or(false)
    }

    async fn ensure_simkl_rewatch(
        &self,
        profile: &Value,
        credentials: &Value,
    ) -> Result<(), String> {
        if !self.profile_flag(profile, "simklTrackRewatches") {
            return Err("Simkl rewatch tracking is turned off".to_owned());
        }
        let key = format!("simkl_account_{}", sanitize_key(str_field(profile, "id")));
        let now = chrono_unix_seconds();
        let cached = self
            .storage
            .read_json(&key)
            .ok()
            .flatten()
            .filter(|cached| {
                cached.get("at").and_then(Value::as_i64).unwrap_or(0) + SIMKL_ACCOUNT_TTL > now
            });
        let plan = match cached {
            Some(cached) => str_field(&cached, "type").to_owned(),
            None => {
                let mut args = credentials.clone();
                args["command"] = json!({"type": "simklAccount"});
                let request = core_value("providerWriteRequests", args)
                    .and_then(|requests| requests.get(0).cloned())
                    .ok_or_else(|| {
                        "Fluxa Core could not plan the Simkl account request".to_owned()
                    })?;
                let body = Self::simkl_get(&http_client()?, &request).await?;
                let plan = body
                    .pointer("/account/type")
                    .and_then(Value::as_str)
                    .unwrap_or("free")
                    .to_owned();
                let _ = self
                    .storage
                    .write_json(&key, &json!({"type": plan, "at": now}));
                plan
            }
        };
        if matches!(plan.as_str(), "pro" | "vip") {
            Ok(())
        } else {
            Err("Simkl rewatch tracking requires Simkl Pro or VIP".to_owned())
        }
    }

    fn rewatch_key(profile_id: &str) -> String {
        format!("simkl_rewatch_{}", sanitize_key(profile_id))
    }

    fn pinned_rewatch_id(&self, profile_id: &str, item_id: &str) -> Option<i64> {
        self.storage
            .read_json(&Self::rewatch_key(profile_id))
            .ok()
            .flatten()?
            .get(item_id)?
            .as_i64()
    }

    fn pin_rewatch_id(&self, profile_id: &str, item_id: &str, rewatch_id: i64) {
        let key = Self::rewatch_key(profile_id);
        let mut pinned = self
            .storage
            .read_json(&key)
            .ok()
            .flatten()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        pinned[item_id] = json!(rewatch_id);
        let _ = self.storage.write_json(&key, &pinned);
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
            "providerIds": meta.get("providerIds"),
            "videoIds": if video_ids.is_empty() { json!([meta_id]) } else { json!(video_ids) },
            "watched": payload.get("watched").and_then(Value::as_bool).unwrap_or(true),
            "addonEpisodes": meta.get("videos"),
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
            args["providerIds"] = payload.get("providerIds").cloned().unwrap_or(Value::Null);
            if provider == "trakt" && str_field(payload, "metaType") != "movie" {
                let item_id = str_field(payload, "itemId");
                let series_id = core_value("traktShowIdFromEpisodeId", json!({"videoId": item_id}))
                    .and_then(|id| id.as_str().map(str::to_owned))
                    .unwrap_or_default();
                let remapped = self
                    .remap_trakt_video_ids(
                        &args,
                        &series_id,
                        &json!([item_id]),
                        &payload["addonEpisodes"],
                    )
                    .await;
                if let Some(id) = remapped.as_ref().and_then(|ids| ids.get(0)) {
                    args["itemId"] = id.clone();
                }
            }
            let Some(plan) = core_value("providerScrobbleRequest", args) else {
                continue;
            };
            match send(&client, &plan).await {
                Ok((status, _)) if (200..300).contains(&status) || status == 409 => {}
                Ok((400, body)) if body.to_string().contains("RATE_LIMIT") => {}
                Ok((status, body)) => {
                    crate::log!("[fluxa-native] {provider} scrobble rejected ({status}): {body}")
                }
                Err(error) => crate::log!("[fluxa-native] {provider} scrobble failed: {error}"),
            }
        }
        Ok(json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn serve(statuses: Vec<u16>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/sync/activities", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for status in statuses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0; 2048];
                let _ = stream.read(&mut buffer);
                let body = "{\"ok\":true}";
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        url
    }

    fn get(statuses: Vec<u16>) -> Result<Value, String> {
        let url = serve(statuses);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let client = http_client()?;
            EffectExecutor::simkl_get(&client, &json!({"method": "GET", "url": url})).await
        })
    }

    #[test]
    fn rate_limited_requests_are_retried() {
        assert_eq!(get(vec![429, 503, 200]).unwrap(), json!({"ok": true}));
    }

    #[test]
    fn revoked_token_is_not_retried() {
        let error = get(vec![401]).unwrap_err();
        assert!(error.contains("sign in again"));
    }

    #[test]
    fn gives_up_after_four_attempts() {
        assert!(get(vec![500, 500, 500, 500]).unwrap_err().contains("500"));
    }
}
