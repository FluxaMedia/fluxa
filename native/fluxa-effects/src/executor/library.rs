use super::*;

impl EffectExecutor {
    pub(super) async fn refresh_continue_watching(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        let profile = match payload.get("profile").filter(|profile| profile.is_object()) {
            Some(profile) => profile.clone(),
            None => self
                .storage
                .read_json("profiles")?
                .and_then(|profiles| {
                    profiles
                        .as_array()?
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
                        .cloned()
                })
                .unwrap_or_else(|| json!({})),
        };
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(profile_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let source = payload.get("source").and_then(Value::as_str);
        let items = self
            .continue_watching_for_source(profile_id, &profile, &prefs, source)
            .await?;
        Ok(json!({"continueWatching": items}))
    }

    pub(super) async fn continue_watching_for_source(
        &self,
        profile_id: &str,
        profile: &Value,
        prefs: &Value,
        source: Option<&str>,
    ) -> Result<Value, String> {
        let nuvio_connected = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .is_some_and(|token| !token.is_empty());
        let requested = source
            .or_else(|| prefs.get("continueWatchingSource")?.as_str())
            .unwrap_or("local");
        let provider = core_value(
            "continueWatchingSourcePlan",
            json!({"source": requested, "nuvioConnected": nuvio_connected}),
        )
        .and_then(|plan| plan.get("provider")?.as_str().map(ToOwned::to_owned));
        match provider.as_deref() {
            Some("nuvio") => {
                let snapshot = self
                    .read_nuvio_library(profile_id, Some(profile).filter(|p| p.is_object()))
                    .await
                    .inspect_err(|error| {
                        crate::log!("[fluxa-native] Nuvio continue watching failed: {error}")
                    })
                    .ok()
                    .flatten();
                return Ok(snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([])));
            }
            Some(other) if providers::is_provider(other) => {
                let snapshot = self
                    .read_provider_library(other, profile_id, profile)
                    .await
                    .inspect_err(|error| {
                        crate::log!("[fluxa-native] {other} continue watching failed: {error}")
                    })
                    .ok();
                return Ok(snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([])));
            }
            Some(other) => {
                crate::log!(
                    "[fluxa-native] continue watching source '{other}' has no native client yet"
                );
                return Ok(json!([]));
            }
            None => {}
        }
        let library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .unwrap_or_else(|| json!({}));
        let progress = library
            .get("progress")
            .cloned()
            .unwrap_or_else(|| json!({}));
        Ok(core_value("buildContinueWatchingFromProgress", progress).unwrap_or_else(|| json!([])))
    }

    pub(super) async fn read_library_state(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let effect_profile = payload.get("profile").filter(|profile| profile.is_object());
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .or_else(|| effect_profile?.get("integrationLibrarySource")?.as_str())
            .unwrap_or("local");
        if source == "nuvio" {
            return self
                .read_nuvio_library(profile_id, effect_profile)
                .await?
                .ok_or_else(|| "Nuvio account is not connected to the active profile".to_owned());
        }
        if providers::is_provider(source) {
            let profile = match effect_profile {
                Some(profile) => profile.clone(),
                None => self.stored_profile(profile_id),
            };
            let snapshot = self
                .read_provider_library(source, profile_id, &profile)
                .await?;
            return core_value("normalizeLibraryDocument", snapshot)
                .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned());
        }
        let stored_library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .or_else(|| self.storage.read_json("library").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let should_derive_continue_watching = stored_library
            .get("continueWatching")
            .is_none_or(Value::is_null);
        let mut library = core_value("normalizeLibraryDocument", stored_library)
            .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned())?;
        if should_derive_continue_watching {
            let progress = library
                .get("progress")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let continue_watching = core_value("buildContinueWatchingFromProgress", progress)
                .ok_or_else(|| "Fluxa Core could not build continue watching items".to_owned())?;
            if let Some(object) = library.as_object_mut() {
                object.insert("continueWatching".to_owned(), continue_watching);
            }
        }
        Ok(library)
    }

    pub(super) fn stored_profile(&self, profile_id: &str) -> Value {
        self.storage
            .read_json("profiles")
            .ok()
            .flatten()
            .and_then(|profiles| {
                profiles
                    .as_array()?
                    .iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
                    .cloned()
            })
            .unwrap_or_else(|| json!({}))
    }

    pub(super) async fn write_library_command(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("local")
            .trim()
            .to_lowercase();
        let provider = providers::is_provider(&source).then_some(source.as_str());
        if provider.is_none() && !matches!(source.as_str(), "" | "local" | "fluxa") {
            return Err(format!(
                "desktop library writes for provider '{source}' are not implemented"
            ));
        }
        let command = payload
            .get("command")
            .filter(|value| value.is_object())
            .ok_or_else(|| "library command is missing".to_owned())?;
        let key = Storage::library_key(profile_id);
        let profile = self.stored_profile(profile_id);
        let library = match provider {
            Some(provider) => match self.cached_provider_library(provider, profile_id) {
                Some(library) => library,
                None => {
                    self.read_provider_library(provider, profile_id, &profile)
                        .await?
                }
            },
            None => self
                .storage
                .read_json(&key)?
                .or_else(|| self.storage.read_json("library").ok().flatten())
                .unwrap_or_else(|| json!({})),
        };
        let library = core_value("normalizeLibraryDocument", library)
            .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned())?;
        let plan = core_value(
            "libraryCommandPlan",
            json!({
                "library": library,
                "command": command,
                "nowIso": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            }),
        )
        .ok_or_else(|| "Fluxa Core could not plan the library command".to_owned())?;
        let updated = plan
            .get("library")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "Fluxa Core returned an invalid library command plan".to_owned())?;
        match provider {
            Some(provider) => {
                let mut remote = command.clone();
                if let Some(id) = command.pointer("/item/id").and_then(Value::as_str) {
                    remote["remove"] = json!(!in_watchlist(&updated, id));
                }
                self.push_provider_command(provider, &profile, &remote)
                    .await?;
                self.store_provider_library(provider, profile_id, &updated);
            }
            None => self.storage.write_json(&key, &updated)?,
        }
        let kind = command
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let result = match kind {
            "toggleWatchlist" => {
                let id = command
                    .pointer("/item/id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let is_in_watchlist = in_watchlist(&updated, id);
                json!({
                    "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                    "isInWatchlist": is_in_watchlist,
                })
            }
            "toggleLibraryStatus" => json!({
                "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                "completed": updated.get("completed").cloned().unwrap_or_else(|| json!([])),
                "dropped": updated.get("dropped").cloned().unwrap_or_else(|| json!([])),
            }),
            "markWatched" => {
                let watched = command
                    .get("watched")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let local_watched_video_ids = command
                    .get("videoIds")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter(|id| {
                        watched
                            && updated
                                .pointer(&format!("/watched/{id}"))
                                .and_then(Value::as_bool)
                                == Some(true)
                    })
                    .collect::<Vec<_>>();
                json!({
                    "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                    "localWatchedVideoIds": local_watched_video_ids,
                })
            }
            _ => return Err(format!("unsupported library command: {kind}")),
        };
        Ok(result)
    }

    pub(super) async fn read_playback_progress(&self, payload: &Value) -> Result<Value, String> {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "playback progress request is missing id".to_owned())?;
        let profile_id = self.active_profile_id(None)?;
        let library = self
            .read_library_state(&json!({"profileId": profile_id}))
            .await?;
        Ok(library
            .get("progress")
            .and_then(|progress| progress.get(id))
            .map(progress_meta)
            .unwrap_or(Value::Null))
    }

    pub(super) async fn read_detail_local_state(&self, payload: &Value) -> Result<Value, String> {
        let profile = payload.get("profile").filter(|value| value.is_object());
        let profile_id = self.active_profile_id(profile)?;
        let library = self
            .read_library_state(&json!({"profileId": profile_id, "profile": profile}))
            .await?;
        let primary_id = payload
            .get("primaryId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let fallback_id = payload.get("fallbackId").and_then(Value::as_str);
        let progress = [Some(primary_id), fallback_id]
            .into_iter()
            .flatten()
            .find_map(|id| library.get("progress").and_then(|items| items.get(id)));
        let ids = [Some(primary_id), fallback_id]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let is_in_watchlist = library
            .get("watchlist")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| ids.contains(&id))
                })
            });
        let local_watched_video_ids = if payload
            .get("contentType")
            .and_then(Value::as_str)
            .is_some_and(|value| matches!(value, "series" | "tv" | "anime"))
        {
            library
                .get("watched")
                .and_then(Value::as_object)
                .map(|watched| {
                    watched
                        .iter()
                        .filter(|(id, value)| {
                            value.as_bool() == Some(true)
                                && ids
                                    .iter()
                                    .any(|series_id| id.starts_with(&format!("{series_id}:")))
                        })
                        .map(|(id, _)| Value::String(id.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let addons = self.load_enabled_addons()?;
        let provider_availability = core_value(
            "providerAvailabilityPlan",
            json!({"addons": addons.clone(), "pluginNames": []}),
        )
        .ok_or_else(|| "Fluxa Core could not resolve stream providers".to_owned())?;
        let feedback = library
            .get("feedback")
            .and_then(Value::as_object)
            .and_then(|values| ids.iter().find_map(|id| values.get(*id)))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(json!({
            "savedPlayback": progress.map(progress_meta).unwrap_or(Value::Null),
            "localWatchedVideoIds": local_watched_video_ids,
            "isInWatchlist": is_in_watchlist,
            "feedback": feedback,
            "hasStreamProviders": provider_availability.get("hasStreamProviders").and_then(Value::as_bool).unwrap_or(false),
            "userAddons": addons
        }))
    }

    pub(super) fn active_profile_id(&self, profile: Option<&Value>) -> Result<String, String> {
        if let Some(id) = profile
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            return Ok(id.to_owned());
        }
        if let Some(id) = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
        {
            return Ok(id);
        }
        Ok("guest".to_owned())
    }

    pub(super) async fn read_nuvio_library(
        &self,
        profile_id: &str,
        effect_profile: Option<&Value>,
    ) -> Result<Option<Value>, String> {
        let Some(base_url) = option_env!("FLUXA_NUVIO_SUPABASE_URL") else {
            return Err("Nuvio API is not configured in this native build".to_owned());
        };
        let Some(api_key) = option_env!("FLUXA_NUVIO_SUPABASE_KEY") else {
            return Err("Nuvio API is not configured in this native build".to_owned());
        };
        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let stored_profile = profiles.as_array().and_then(|items| {
            items
                .iter()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
        });
        let Some(profile) = effect_profile.or(stored_profile) else {
            return Err(format!(
                "Nuvio library could not find the active profile ({profile_id})"
            ));
        };
        let Some(mut token) = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
        else {
            return Err(
                "Nuvio library is selected, but this profile has no Nuvio session".to_owned(),
            );
        };

        let client = Client::builder()
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let expires_at = profile
            .get("nuvioTokenExpiresAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if expires_at <= chrono_unix_seconds() + 60 {
            if let Some(refresh_token) = profile
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
                    .map_err(|error| format!("Nuvio session refresh failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio session refresh failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| {
                        format!("Nuvio session refresh response was invalid: {error}")
                    })?;
                if let Some(access_token) = refreshed.get("access_token").and_then(Value::as_str) {
                    token = access_token.to_owned();
                    let mut updated_profile = profile.clone();
                    if let Some(object) = updated_profile.as_object_mut() {
                        object.insert("nuvioAccessToken".into(), json!(access_token));
                        if let Some(refresh) = refreshed.get("refresh_token") {
                            object.insert("nuvioRefreshToken".into(), refresh.clone());
                        }
                        let expires_in = refreshed
                            .get("expires_in")
                            .and_then(Value::as_i64)
                            .unwrap_or(3600);
                        object.insert(
                            "nuvioTokenExpiresAt".into(),
                            json!(chrono_unix_seconds() + expires_in),
                        );
                    }
                    if let Some(items) = profiles.as_array() {
                        let updated = items
                            .iter()
                            .map(|item| {
                                if item.get("id").and_then(Value::as_str) == Some(profile_id) {
                                    updated_profile.clone()
                                } else {
                                    item.clone()
                                }
                            })
                            .collect::<Vec<_>>();
                        self.storage.write_json("profiles", &json!(updated))?;
                    }
                }
            }
        }
        let nuvio_profile_id = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
            .unwrap_or(1);
        let endpoint = format!("{}/rest/v1/rpc/", base_url.trim_end_matches('/'));
        let headers = |request: reqwest::RequestBuilder| {
            request.header("apikey", api_key).bearer_auth(&token)
        };
        let (library, progress) = futures::try_join!(
            async {
                headers(client.post(format!("{endpoint}sync_pull_library")))
                    .json(&json!({"p_profile_id": nuvio_profile_id, "p_limit": 500, "p_offset": 0}))
                    .send()
                    .await
                    .map_err(|error| format!("Nuvio library request failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio library request failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| format!("Nuvio library response was invalid: {error}"))
            },
            async {
                headers(client.post(format!("{endpoint}sync_pull_watch_progress")))
                    .json(&json!({"p_profile_id": nuvio_profile_id, "p_limit": 1000}))
                    .send()
                    .await
                    .map_err(|error| format!("Nuvio watch progress request failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio watch progress request failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| format!("Nuvio watch progress response was invalid: {error}"))
            }
        )?;
        crate::log!(
            "[fluxa-native] Nuvio library sync: library_rows={} progress_rows={}",
            library.as_array().map_or(0, Vec::len),
            progress.as_array().map_or(0, Vec::len)
        );
        core_value(
            "nuvioProviderLibrarySnapshot",
            json!({"library": library, "progress": progress}),
        )
        .map(Some)
        .ok_or_else(|| "Fluxa Core could not build the Nuvio library snapshot".to_owned())
    }
}

pub(super) fn in_watchlist(library: &Value, id: &str) -> bool {
    library
        .get("watchlist")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
        })
}

pub(super) fn progress_meta(progress: &Value) -> Value {
    let mut meta = progress
        .get("meta")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(values) = progress.as_object() {
        for (key, value) in values {
            if key != "meta" {
                meta.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(meta)
}
