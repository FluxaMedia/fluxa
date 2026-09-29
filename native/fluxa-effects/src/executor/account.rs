use super::*;

const REMOTE_CHECK_SECONDS: i64 = 30;

pub(super) struct NuvioSession {
    pub client: Client,
    pub base_url: &'static str,
    pub api_key: &'static str,
    pub token: String,
    pub profile_index: i64,
}

impl NuvioSession {
    pub fn rpc(&self, name: &str) -> reqwest::RequestBuilder {
        self.client
            .post(format!(
                "{}/rest/v1/rpc/{name}",
                self.base_url.trim_end_matches('/')
            ))
            .header("apikey", self.api_key)
            .bearer_auth(&self.token)
    }

    pub fn rest(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(format!(
                "{}/rest/v1/{path}",
                self.base_url.trim_end_matches('/')
            ))
            .header("apikey", self.api_key)
            .bearer_auth(&self.token)
    }
}

impl EffectExecutor {
    pub(super) fn active_profile(&self) -> Result<(String, Value, Value), String> {
        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "guest".to_owned());
        let profile = find_profile(&profiles, &active_id);
        Ok((active_id, profile, profiles))
    }

    pub(super) async fn nuvio_session(
        &self,
        profile: &Value,
    ) -> Result<Option<NuvioSession>, String> {
        let (Some(base_url), Some(api_key)) = (
            option_env!("FLUXA_NUVIO_SUPABASE_URL"),
            option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
        ) else {
            return Ok(None);
        };
        let Some(mut token) = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
        else {
            return Ok(None);
        };
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let expires_at = profile
            .get("nuvioTokenExpiresAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if expires_at <= chrono_unix_seconds() + 60
            && let Some(refresh_token) = profile
                .get("nuvioRefreshToken")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
        {
            let refreshed = client
                .post(format!(
                    "{}/auth/v1/token?grant_type=refresh_token",
                    base_url.trim_end_matches('/')
                ))
                .header("apikey", api_key)
                .json(&json!({"refresh_token": refresh_token}))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| format!("Nuvio session refresh failed: {error}"))?
                .json::<Value>()
                .await
                .map_err(|error| format!("Nuvio session refresh response was invalid: {error}"))?;
            if let Some(access_token) = refreshed.get("access_token").and_then(Value::as_str) {
                token = access_token.to_owned();
                self.store_nuvio_tokens(profile, &refreshed)?;
            }
        }
        let profile_index = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .filter(|index| *index > 0)
            .unwrap_or(1);
        Ok(Some(NuvioSession {
            client,
            base_url,
            api_key,
            token,
            profile_index,
        }))
    }

    fn store_nuvio_tokens(&self, profile: &Value, refreshed: &Value) -> Result<(), String> {
        let Some(id) = profile.get("id").and_then(Value::as_str) else {
            return Ok(());
        };
        let mut profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let Some(stored) = profiles.as_array_mut().and_then(|items| {
            items
                .iter_mut()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
        }) else {
            return Ok(());
        };
        let Some(fields) = stored.as_object_mut() else {
            return Ok(());
        };
        fields.insert("nuvioAccessToken".into(), refreshed["access_token"].clone());
        if let Some(refresh) = refreshed.get("refresh_token") {
            fields.insert("nuvioRefreshToken".into(), refresh.clone());
        }
        let expires_in = refreshed
            .get("expires_in")
            .and_then(Value::as_i64)
            .unwrap_or(3600);
        fields.insert(
            "nuvioTokenExpiresAt".into(),
            json!(chrono_unix_seconds() + expires_in),
        );
        self.storage.write_json("profiles", &profiles)
    }

    pub(super) async fn account_addons(&self) -> Result<Value, String> {
        let (active_id, profile, profiles) = self.active_profile()?;
        self.addons_for_profile(&profiles, &active_id, &profile)
            .await
    }

    pub(super) async fn addons_for_profile(
        &self,
        profiles: &Value,
        active_id: &str,
        profile: &Value,
    ) -> Result<Value, String> {
        let source = account_source(profile, "addons");
        let addons = match source.get("backend").and_then(Value::as_str) {
            Some("nuvio") => self.nuvio_addons(profile, &source).await?,
            _ => self.local_addons(profiles, active_id)?,
        };
        normalize_enabled_addons(addons)
    }

    fn local_addons(&self, profiles: &Value, active_id: &str) -> Result<Value, String> {
        let owner = core_value(
            "effectiveAddonsOwnerId",
            json!({"profiles": profiles, "activeProfileId": active_id}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| active_id.to_owned());
        Ok(self
            .storage
            .read_json(&Storage::addons_key(&owner))?
            .or_else(|| {
                self.storage
                    .read_json(&Storage::addons_key(active_id))
                    .ok()
                    .flatten()
            })
            .or_else(|| self.storage.read_json("addons").ok().flatten())
            .filter(Value::is_array)
            .unwrap_or_else(|| json!([])))
    }

    async fn nuvio_addons(&self, profile: &Value, source: &Value) -> Result<Value, String> {
        let Some(key) = source.get("snapshotKey").and_then(Value::as_str) else {
            return Ok(json!([]));
        };
        let snapshot = self.storage.read_json(key)?.unwrap_or(Value::Null);
        let checked_at = snapshot
            .get("checkedAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if chrono_unix_seconds() - checked_at < REMOTE_CHECK_SECONDS {
            return Ok(snapshot_addons(&snapshot));
        }
        match self.refresh_nuvio_addons(profile, &snapshot).await {
            Ok(fresh) => {
                self.storage.write_json(key, &fresh)?;
                Ok(snapshot_addons(&fresh))
            }
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio add-on refresh failed: {error}");
                Ok(snapshot_addons(&snapshot))
            }
        }
    }

    async fn refresh_nuvio_addons(
        &self,
        profile: &Value,
        snapshot: &Value,
    ) -> Result<Value, String> {
        let session = self
            .nuvio_session(profile)
            .await?
            .ok_or_else(|| "no Nuvio session".to_owned())?;
        let scope = self.nuvio_addons_profile_index(&session).await;
        let rows = session
            .rest(&format!(
                "addons?select=*&profile_id=eq.{scope}&order=sort_order"
            ))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("add-on list request failed: {error}"))?
            .json::<Value>()
            .await
            .map_err(|error| format!("add-on list response invalid: {error}"))?;
        let plan = |manifests: &Value| {
            core_value(
                "nuvioAddonSnapshotPlan",
                json!({"rows": rows, "snapshot": snapshot, "manifests": manifests}),
            )
            .ok_or_else(|| "Fluxa Core could not plan the Nuvio add-on snapshot".to_owned())
        };
        let mut result = plan(&Value::Null)?;
        let fetch = result
            .get("fetch")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !fetch.is_empty() {
            let client = &session.client;
            let fetched = futures::future::join_all(fetch.iter().filter_map(Value::as_str).map(
                |url| async move {
                    let manifest = client
                        .get(url)
                        .send()
                        .await
                        .and_then(reqwest::Response::error_for_status)
                        .map_err(|error| error.to_string())?
                        .json::<Value>()
                        .await
                        .map_err(|error| error.to_string());
                    if let Err(error) = &manifest {
                        crate::log!(
                            "[fluxa-native] Nuvio add-on manifest failed for {}: {error}",
                            url_without_query(url)
                        );
                    }
                    manifest.map(|manifest| (url.to_owned(), manifest))
                },
            ))
            .await;
            let manifests = fetched
                .into_iter()
                .flatten()
                .collect::<serde_json::Map<_, _>>();
            result = plan(&Value::Object(manifests))?;
        }
        Ok(json!({
            "checkedAt": chrono_unix_seconds(),
            "rows": result["rows"],
            "addons": result["addons"],
        }))
    }

    pub(super) async fn nuvio_synced(
        &self,
        session: &NuvioSession,
        profile: &Value,
        resource: &str,
    ) -> Result<Value, String> {
        let source = account_source(profile, resource);
        let key = source
            .get("snapshotKey")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("no Nuvio snapshot for {resource}"))?;
        let state = self.storage.read_json(key)?.unwrap_or(Value::Null);
        let checked_at = state
            .get("checkedAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if chrono_unix_seconds() - checked_at < REMOTE_CHECK_SECONDS {
            return Ok(state);
        }
        match self.pull_nuvio_delta(session, resource, &state).await {
            Ok(mut fresh) => {
                fresh["checkedAt"] = json!(chrono_unix_seconds());
                self.storage.write_json(key, &fresh)?;
                Ok(fresh)
            }
            Err(error) if state.get("initialized") == Some(&Value::Bool(true)) => {
                crate::log!("[fluxa-native] Nuvio {resource} refresh failed: {error}");
                Ok(state)
            }
            Err(error) => Err(error),
        }
    }

    pub(super) async fn push_nuvio_write(
        &self,
        profile: &Value,
        action: &Value,
    ) -> Result<(), String> {
        let session = self
            .nuvio_session(profile)
            .await?
            .ok_or_else(|| "Nuvio is not connected to the active profile".to_owned())?;
        let requests = core_value(
            "nuvioWriteRequests",
            json!({
                "profileIndex": session.profile_index,
                "action": action,
                "nowMs": chrono::Utc::now().timestamp_millis(),
            }),
        )
        .unwrap_or_else(|| json!([]));
        for request in requests.as_array().into_iter().flatten() {
            let rpc = request
                .get("rpc")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let response = session
                .rpc(rpc)
                .json(request.get("body").unwrap_or(&Value::Null))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(format!("Nuvio rejected the change ({status}): {body}"));
            }
        }
        for resource in ["library", "progress", "history"] {
            if let Some(key) = account_source(profile, resource)
                .get("snapshotKey")
                .and_then(Value::as_str)
                && let Some(mut state) = self.storage.read_json(key)?
            {
                state["checkedAt"] = json!(0);
                self.storage.write_json(key, &state)?;
            }
        }
        Ok(())
    }

    async fn pull_nuvio_delta(
        &self,
        session: &NuvioSession,
        resource: &str,
        state: &Value,
    ) -> Result<Value, String> {
        let rpc_name = match resource {
            "library" => "library",
            "progress" => "watch_progress",
            _ => "watched_items",
        };
        let call = |name: String, body: Value| async move {
            session
                .rpc(&name)
                .json(&body)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| format!("Nuvio {name} failed: {error}"))?
                .json::<Value>()
                .await
                .map_err(|error| format!("Nuvio {name} response was invalid: {error}"))
        };
        let profile_id = session.profile_index;
        let plan = core_value("nuvioDeltaSyncRequestPlan", json!({"state": state}))
            .unwrap_or_else(|| json!({"mode": "bootstrap", "cursor": 0}));
        let mut request = json!({"resource": resource, "state": state});
        let mut cursor = plan.get("cursor").and_then(Value::as_i64).unwrap_or(0);
        if plan.get("mode").and_then(Value::as_str) != Some("delta") {
            cursor = call(
                format!("sync_get_{rpc_name}_delta_cursor"),
                json!({"p_profile_id": profile_id}),
            )
            .await?
            .as_i64()
            .unwrap_or(0);
            let mut snapshot = Vec::new();
            match resource {
                "progress" => {
                    let rows = call(
                        "sync_pull_watch_progress".to_owned(),
                        json!({"p_profile_id": profile_id, "p_limit": 200}),
                    )
                    .await?;
                    snapshot.extend(rows.as_array().cloned().unwrap_or_default());
                }
                _ => {
                    for page in 0.. {
                        let body = if resource == "library" {
                            json!({"p_profile_id": profile_id, "p_limit": 500, "p_offset": page * 500})
                        } else {
                            json!({"p_profile_id": profile_id, "p_page": page + 1, "p_page_size": 500})
                        };
                        let rows = call(format!("sync_pull_{rpc_name}"), body)
                            .await?
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                        let full = rows.len() == 500;
                        snapshot.extend(rows);
                        if !full {
                            break;
                        }
                    }
                }
            }
            request["snapshot"] = Value::Array(snapshot);
            request["snapshotCursor"] = json!(cursor);
        }
        let mut events = Vec::new();
        loop {
            let page = call(
                format!("sync_pull_{rpc_name}_delta"),
                json!({"p_profile_id": profile_id, "p_since_event_id": cursor, "p_limit": 1000}),
            )
            .await?
            .as_array()
            .cloned()
            .unwrap_or_default();
            let full = page.len() == 1000;
            cursor = page
                .iter()
                .filter_map(|event| event.get("event_id").and_then(Value::as_i64))
                .max()
                .unwrap_or(cursor);
            events.extend(page);
            if !full {
                break;
            }
        }
        request["events"] = Value::Array(events);
        let method = if resource == "progress" {
            "nuvioApplyProgressSync"
        } else {
            "nuvioApplyDeltaSync"
        };
        core_value(method, request)
            .ok_or_else(|| format!("Fluxa Core could not apply the Nuvio {resource} delta"))
    }

    pub(super) async fn nuvio_rows(
        &self,
        session: &NuvioSession,
        profile: &Value,
        entity: &str,
        rpc: &str,
        body: Value,
    ) -> Result<Value, String> {
        let source = account_source(profile, entity);
        let key = source
            .get("snapshotKey")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("no Nuvio snapshot for {entity}"))?;
        let state = self.storage.read_json(key)?.unwrap_or(Value::Null);
        let checked_at = state
            .get("checkedAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if chrono_unix_seconds() - checked_at < REMOTE_CHECK_SECONDS {
            return Ok(state["rows"].clone());
        }
        let pulled = async {
            session
                .rpc(rpc)
                .json(&body)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| format!("Nuvio {rpc} failed: {error}"))?
                .json::<Value>()
                .await
                .map_err(|error| format!("Nuvio {rpc} response was invalid: {error}"))
        }
        .await;
        match pulled {
            Ok(rows) => {
                self.storage.write_json(
                    key,
                    &json!({"checkedAt": chrono_unix_seconds(), "rows": rows}),
                )?;
                Ok(rows)
            }
            Err(error) if state.get("rows").is_some() => {
                crate::log!("[fluxa-native] Nuvio {entity} refresh failed: {error}");
                Ok(state["rows"].clone())
            }
            Err(error) => Err(error),
        }
    }

    pub(super) async fn nuvio_home_catalog_items(
        &self,
        session: &NuvioSession,
        profile: &Value,
    ) -> Option<Value> {
        for platform in ["mobile", "tv"] {
            let rows = self
                .nuvio_rows(
                    session,
                    profile,
                    &format!("home_catalogs_{platform}"),
                    "sync_pull_home_catalog_settings",
                    json!({"p_profile_id": session.profile_index, "p_platform": platform}),
                )
                .await
                .inspect_err(|error| crate::log!("[fluxa-native] {error}"))
                .ok()?;
            if let Some(items) = rows
                .as_array()
                .and_then(|rows| rows.first())
                .and_then(|row| row.get("settings_json")?.get("items"))
                .filter(|items| items.as_array().is_some_and(|items| !items.is_empty()))
            {
                return Some(items.clone());
            }
        }
        None
    }

    pub(super) async fn refresh_nuvio_profiles(&self, session: &NuvioSession, profile: &Value) {
        let checked_at = self
            .storage
            .read_json("nuvio_profiles_checked_at")
            .ok()
            .flatten()
            .and_then(|value| value.as_i64())
            .unwrap_or_default();
        if chrono_unix_seconds() - checked_at < 600 {
            return;
        }
        let _ = self
            .storage
            .write_json("nuvio_profiles_checked_at", &json!(chrono_unix_seconds()));
        let remote = match session
            .rpc("sync_pull_profiles")
            .json(&json!({}))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => response.json::<Value>().await.unwrap_or(Value::Null),
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio profiles request failed: {error}");
                return;
            }
        };
        let catalog = match session
            .rpc("get_avatar_catalog")
            .json(&json!({}))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                response.json::<Value>().await.unwrap_or(Value::Null)
            }
            _ => Value::Null,
        };
        let Ok(Some(profiles)) = self.storage.read_json("profiles") else {
            return;
        };
        if let Some(next) = core_value(
            "nuvioApplyRemoteProfiles",
            json!({
                "sessionProfile": profile,
                "profiles": profiles,
                "nuvioProfiles": remote,
                "avatarCatalog": catalog,
            }),
        )
        .filter(Value::is_array)
        {
            let _ = self.storage.write_json("profiles", &next);
        }
    }

    async fn nuvio_addons_profile_index(&self, session: &NuvioSession) -> i64 {
        if session.profile_index == 1 {
            return 1;
        }
        let remote_profiles = match session
            .rpc("sync_pull_profiles")
            .json(&json!({}))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => response.json::<Value>().await.unwrap_or(Value::Null),
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio profile scopes request failed: {error}");
                return session.profile_index;
            }
        };
        core_value(
            "nuvioEffectiveProfileScopes",
            json!({"profileIndex": session.profile_index, "profiles": remote_profiles}),
        )
        .and_then(|scopes| scopes.get("addons").and_then(Value::as_i64))
        .unwrap_or(session.profile_index)
    }
}

pub fn account_source(profile: &Value, entity: &str) -> Value {
    core_value(
        "accountSource",
        json!({"profile": profile, "entity": entity}),
    )
    .unwrap_or_else(|| json!({"backend": "local"}))
}

pub fn snapshot_addons(snapshot: &Value) -> Value {
    snapshot
        .get("addons")
        .filter(|addons| addons.is_array())
        .cloned()
        .unwrap_or_else(|| json!([]))
}

pub(super) fn find_profile(profiles: &Value, id: &str) -> Value {
    profiles
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
        })
        .cloned()
        .unwrap_or_else(|| json!({}))
}
