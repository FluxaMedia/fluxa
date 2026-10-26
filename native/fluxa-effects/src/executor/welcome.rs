use super::*;

const TV_LOGIN_URL: &str = "https://nuvio.tv/tv-login";
const DEVICE_LOGIN_URL: &str = "https://nuvio.tv/link";
const UNKNOWN: &str = "auth.error.unknown";
pub const OFFLINE: &str = "offline";

fn supabase() -> Result<(&'static str, &'static str), String> {
    match (
        option_env!("FLUXA_NUVIO_SUPABASE_URL"),
        option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
    ) {
        (Some(url), Some(key)) => Ok((url.trim_end_matches('/'), key)),
        _ => Err(UNKNOWN.to_owned()),
    }
}

fn client() -> Result<Client, String> {
    Client::builder()
        .bundled_roots()
        .user_agent("Fluxa/1.0")
        .native_timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| UNKNOWN.to_owned())
}

fn error_key(status: u16, body: &str) -> String {
    core_value("nuvioAuthError", json!({"status": status, "body": body}))
        .and_then(|key| key.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| UNKNOWN.to_owned())
}

async fn send(
    client: &Client,
    url: String,
    key: &str,
    body: &Value,
) -> Result<String, (u16, String)> {
    let response = client
        .post(url)
        .header("apikey", key)
        .json(body)
        .send()
        .await
        .map_err(|_| (0, String::new()))?;
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    if (200..300).contains(&status) {
        Ok(text)
    } else {
        Err((status, text))
    }
}

async fn post(
    client: &Client,
    url: String,
    key: &str,
    body: &Value,
) -> Result<(u16, String), String> {
    send(client, url, key, body)
        .await
        .map(|text| (200, text))
        .map_err(|(status, text)| error_key(status, &text))
}

fn json_body(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|_| UNKNOWN.to_owned())
}

fn nonce() -> String {
    super::providers::random_hex()
        .map(|hex| hex[..48].to_owned())
        .unwrap_or_else(|| chrono_unix_seconds().to_string())
}

impl EffectExecutor {
    pub fn nuvio_sign_in(
        &self,
        email: String,
        password: String,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            let (base, key) = supabase()?;
            let (_, text) = post(
                &client()?,
                format!("{base}/auth/v1/token?grant_type=password"),
                key,
                &json!({"email": email, "password": password}),
            )
            .await?;
            executor
                .finish_nuvio_login(&json_body(&text)?, &email)
                .await
        })
    }

    pub fn nuvio_avatar_catalog(&self) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        super::detached(async move {
            let (base, key) = supabase()?;
            let text = send(
                &client()?,
                format!("{base}/rest/v1/rpc/get_avatar_catalog"),
                key,
                &json!({}),
            )
            .await
            .map_err(|(status, body)| error_key(status, &body))?;
            let catalog = json_body(&text)?;
            core_value(
                "nuvioAvatarCatalogPack",
                json!({
                    "catalog": catalog,
                    "storageBase": format!("{base}/storage/v1/object/public/avatars/"),
                }),
            )
            .ok_or_else(|| UNKNOWN.to_owned())
        })
    }

    pub fn nuvio_tv_start(&self) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        super::detached(async move {
            let (base, key) = supabase()?;
            let client = client()?;
            let nonce = nonce();
            let device = send(
                &client,
                format!("{base}/rest/v1/rpc/start_device_login_session"),
                key,
                &json!({
                    "p_device_nonce": nonce,
                    "p_redirect_base_url": DEVICE_LOGIN_URL,
                    "p_device_type": "tv",
                    "p_device_name": "Fluxa",
                }),
            )
            .await;
            let (text, code_field, url_field) = match device {
                Ok(text) => (text, "device_code", "verification_uri_complete"),
                Err((_, body))
                    if body.contains("start_device_login_session")
                        && body.to_lowercase().contains("could not find the function") =>
                {
                    let (_, text) = post(
                        &client,
                        format!("{base}/rest/v1/rpc/start_tv_login_session"),
                        key,
                        &json!({"p_device_nonce": nonce, "p_redirect_base_url": TV_LOGIN_URL}),
                    )
                    .await?;
                    (text, "code", "web_url")
                }
                Err((status, body)) => return Err(error_key(status, &body)),
            };
            let started = json_body(&text)?;
            let row = started
                .as_array()
                .and_then(|rows| rows.first())
                .unwrap_or(&started);
            let text_of = |field: &str| row.get(field).and_then(Value::as_str).unwrap_or_default();
            let expires_at = chrono::DateTime::parse_from_rfc3339(text_of("expires_at"))
                .map(|time| time.timestamp())
                .unwrap_or_else(|_| chrono_unix_seconds() + 300);
            if text_of(code_field).is_empty() || text_of(url_field).is_empty() {
                return Err(UNKNOWN.to_owned());
            }
            Ok(json!({
                "code": text_of(code_field),
                "webUrl": text_of(url_field),
                "userCode": if code_field == "device_code" { text_of("user_code") } else { text_of(code_field) },
                "verificationUri": if code_field == "device_code" { text_of("verification_uri") } else { TV_LOGIN_URL },
                "expiresAt": expires_at,
                "interval": row.get("poll_interval_seconds").and_then(Value::as_u64).unwrap_or(3).max(2),
                "nonce": nonce,
            }))
        })
    }

    pub fn nuvio_tv_poll(
        &self,
        code: String,
        nonce: String,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            let (base, key) = supabase()?;
            let client = client()?;
            let (_, text) = post(
                &client,
                format!("{base}/rest/v1/rpc/poll_tv_login_session"),
                key,
                &json!({"p_code": code, "p_device_nonce": nonce}),
            )
            .await?;
            let polled = json_body(&text)?;
            let row = polled
                .as_array()
                .and_then(|rows| rows.first())
                .unwrap_or(&polled);
            let approved = row
                .get("approved")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || row
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| status.eq_ignore_ascii_case("approved"));
            if !approved {
                let ended = row
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| {
                        ["expired", "used", "cancelled"]
                            .iter()
                            .any(|ended| status.eq_ignore_ascii_case(ended))
                    });
                return Ok(json!({"state": if ended { "ended" } else { "pending" }}));
            }
            let (_, text) = post(
                &client,
                format!("{base}/functions/v1/tv-logins-exchange"),
                key,
                &json!({"code": code, "device_nonce": nonce}),
            )
            .await?;
            let session = json_body(&text)?;
            let profile = executor.finish_nuvio_login(&session, "").await?;
            Ok(json!({"state": "success", "profile": profile}))
        })
    }

    async fn finish_nuvio_login(&self, session: &Value, email: &str) -> Result<Value, String> {
        let id = nonce().chars().take(32).collect::<String>();
        let profile = core_value(
            "nuvioSessionProfile",
            json!({
                "id": id,
                "email": email,
                "nowSeconds": chrono_unix_seconds(),
                "session": session,
            }),
        )
        .filter(|profile| !profile.is_null())
        .ok_or_else(|| UNKNOWN.to_owned())?;
        self.save_profile(&profile);
        let _ = self
            .storage
            .write_json("nuvio_profiles_checked_at", &json!(0));
        if let Ok(Some(nuvio)) = self.nuvio_session(&profile).await {
            self.refresh_nuvio_profiles(&nuvio, &profile).await;
        }
        let stored = self.storage.read_json("profiles").ok().flatten();
        Ok(stored
            .as_ref()
            .map(|profiles| find_profile(profiles, &id))
            .filter(|found| found.get("id").is_some())
            .unwrap_or(profile))
    }

    pub fn nuvio_verify_pin(
        &self,
        profile: Value,
        pin: String,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            let (_, text) = executor
                .nuvio_pin_rpc(
                    &profile,
                    "verify_profile_pin",
                    |index| json!({"p_profile_id": index, "p_pin": pin}),
                )
                .await
                .map_err(|failure| match failure {
                    PinRpcFailure::Offline => OFFLINE.to_owned(),
                    PinRpcFailure::Rejected(status, body) => error_key(status, &body),
                })?;
            let body = json_body(&text)?;
            let result = body
                .as_array()
                .and_then(|rows| rows.first())
                .cloned()
                .unwrap_or(body);
            if result.get("unlocked").and_then(Value::as_bool) == Some(true) {
                executor.resync_nuvio_profiles(&profile).await;
                executor.cache_nuvio_pin(&profile, &pin);
            }
            Ok(result)
        })
    }

    pub fn nuvio_set_pin(
        &self,
        profile: Value,
        pin: String,
        current: Option<String>,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            executor
                .push_nuvio_profile(&profile, false)
                .await
                .map_err(pin_change_error)?;
            executor
                .nuvio_pin_rpc(&profile, "set_profile_pin", |index| {
                    let mut body = json!({"p_profile_id": index, "p_pin": pin});
                    if let Some(current) = &current {
                        body["p_current_pin"] = json!(current);
                    }
                    body
                })
                .await
                .map_err(pin_change_error)?;
            executor.resync_nuvio_profiles(&profile).await;
            executor.cache_nuvio_pin(&profile, &pin);
            Ok(json!({}))
        })
    }

    pub fn nuvio_clear_pin(
        &self,
        profile: Value,
        current: Option<String>,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            executor
                .push_nuvio_profile(&profile, false)
                .await
                .map_err(pin_change_error)?;
            executor
                .nuvio_pin_rpc(&profile, "clear_profile_pin", |index| {
                    let mut body = json!({"p_profile_id": index});
                    if let Some(current) = &current {
                        body["p_current_pin"] = json!(current);
                    }
                    body
                })
                .await
                .map_err(pin_change_error)?;
            executor.resync_nuvio_profiles(&profile).await;
            Ok(json!({}))
        })
    }

    pub fn nuvio_push_profile(
        &self,
        profile: Value,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            executor
                .push_nuvio_profile(&profile, false)
                .await
                .map_err(pin_change_error)?;
            executor.resync_nuvio_profiles(&profile).await;
            Ok(json!({}))
        })
    }

    async fn push_nuvio_profile(&self, profile: &Value, create: bool) -> Result<i64, PinRpcFailure> {
        let session = match self.nuvio_session(profile).await {
            Ok(Some(session)) => session,
            _ => return Err(PinRpcFailure::Offline),
        };
        let remote = session
            .rpc("sync_pull_profiles")
            .json(&json!({}))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| PinRpcFailure::Offline)?
            .json::<Value>()
            .await
            .map_err(|_| PinRpcFailure::Offline)?;
        let catalog = match session.rpc("get_avatar_catalog").json(&json!({})).send().await {
            Ok(response) if response.status().is_success() => {
                response.json::<Value>().await.unwrap_or(Value::Null)
            }
            _ => Value::Null,
        };
        let mut args = json!({
            "profile": profile,
            "nuvioProfiles": remote,
            "avatarCatalog": catalog,
        });
        if !create {
            args["profileIndex"] = json!(session.profile_index);
        }
        let Some(plan) = core_value("nuvioProfilePushPlan", args) else {
            return Err(PinRpcFailure::Rejected(0, String::new()));
        };
        let index = plan["profileIndex"].as_i64().unwrap_or(session.profile_index);
        let response = session
            .rpc("sync_push_profiles")
            .json(&plan["body"])
            .send()
            .await
            .map_err(|_| PinRpcFailure::Offline)?;
        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            Ok(index)
        } else {
            Err(PinRpcFailure::Rejected(
                status,
                response.text().await.unwrap_or_default(),
            ))
        }
    }

    pub fn nuvio_create_profile(
        &self,
        profile: Value,
        pin: Option<String>,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            let index = executor
                .push_nuvio_profile(&profile, true)
                .await
                .map_err(pin_change_error)?;
            if let Some(pin) = pin
                && let Ok(Some(session)) = executor.nuvio_session(&profile).await
            {
                let _ = session
                    .rpc("set_profile_pin")
                    .json(&json!({"p_profile_id": index, "p_pin": pin}))
                    .send()
                    .await;
            }
            executor.resync_nuvio_profiles(&profile).await;
            Ok(json!({}))
        })
    }

    pub fn nuvio_delete_profile(
        &self,
        profile: Value,
    ) -> std::sync::mpsc::Receiver<Result<Value, String>> {
        let executor = self.clone();
        super::detached(async move {
            let session = match executor.nuvio_session(&profile).await {
                Ok(Some(session)) => session,
                _ => return Err(OFFLINE.to_owned()),
            };
            let response = session
                .rpc("sync_delete_profile_data")
                .json(&json!({"p_profile_id": session.profile_index}))
                .send()
                .await
                .map_err(|_| "profiles.pin_requires_internet".to_owned())?;
            if !response.status().is_success() {
                return Err("profiles.sync_failed".to_owned());
            }
            executor.resync_nuvio_profiles(&profile).await;
            Ok(json!({}))
        })
    }

    async fn nuvio_pin_rpc(
        &self,
        profile: &Value,
        name: &str,
        body: impl FnOnce(i64) -> Value,
    ) -> Result<(u16, String), PinRpcFailure> {
        let session = match self.nuvio_session(profile).await {
            Ok(Some(session)) => session,
            _ => return Err(PinRpcFailure::Offline),
        };
        let response = session
            .rpc(name)
            .json(&body(session.profile_index))
            .send()
            .await
            .map_err(|_| PinRpcFailure::Offline)?;
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        if (200..300).contains(&status) {
            Ok((status, text))
        } else {
            Err(PinRpcFailure::Rejected(status, text))
        }
    }

    async fn resync_nuvio_profiles(&self, profile: &Value) {
        let _ = self
            .storage
            .write_json("nuvio_profiles_checked_at", &json!(0));
        if let Ok(Some(nuvio)) = self.nuvio_session(profile).await {
            self.refresh_nuvio_profiles(&nuvio, profile).await;
        }
    }

    fn cache_nuvio_pin(&self, profile: &Value, pin: &str) {
        let Some(id) = profile.get("id").and_then(Value::as_str) else {
            return;
        };
        let Ok(Some(mut profiles)) = self.storage.read_json("profiles") else {
            return;
        };
        let Some(salt) = super::providers::random_hex() else {
            return;
        };
        let Some(stored) = profiles.as_array_mut().and_then(|items| {
            items
                .iter_mut()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
        }) else {
            return;
        };
        if let Some(next) = core_value(
            "nuvioPinCache",
            json!({"profile": stored.clone(), "salt": &salt[..16], "pin": pin}),
        ) {
            *stored = next;
            let _ = self.storage.write_json("profiles", &profiles);
        }
    }
}

enum PinRpcFailure {
    Offline,
    Rejected(u16, String),
}

fn pin_change_error(failure: PinRpcFailure) -> String {
    match failure {
        PinRpcFailure::Offline => "profiles.pin_requires_internet".to_owned(),
        PinRpcFailure::Rejected(..) => "profiles.sync_failed".to_owned(),
    }
}
