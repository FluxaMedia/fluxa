use super::*;
use futures::StreamExt;

const BLOCKING_PROGRESS_METAS: usize = 12;
const META_FETCH_CONCURRENCY: usize = 12;
const SHARED_LIBRARY_READ: std::time::Duration = std::time::Duration::from_millis(1500);

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
        let items = self
            .raw_continue_watching(profile_id, profile, prefs, source)
            .await?;
        Ok(self.annotate_episode_status(profile_id, items).await)
    }

    async fn annotate_episode_status(&self, profile_id: &str, mut items: Value) -> Value {
        const TTL_MS: i64 = 6 * 60 * 60 * 1000;
        const MAX_FETCHES: usize = 12;
        let Some(list) = items.as_array_mut() else {
            return items;
        };
        let now_ms = chrono::Utc::now().timestamp_millis();
        let key = format!("cw_episodes_{profile_id}");
        let cache = self
            .storage
            .read_json(&key)
            .ok()
            .flatten()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        let stale: Vec<(String, String)> = list
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("series"))
            .filter_map(|item| {
                let id = item.get("id")?.as_str()?.to_owned();
                let fresh = cache
                    .get(&id)
                    .and_then(|entry| entry.get("at")?.as_i64())
                    .is_some_and(|at| now_ms - at < TTL_MS);
                (!fresh).then(|| (id, "series".to_owned()))
            })
            .take(MAX_FETCHES)
            .collect();
        if !stale.is_empty() {
            let executor = self.clone();
            let task = async move {
                let fetched: Vec<_> = futures::stream::iter(stale.into_iter().map(|(id, kind)| {
                    let executor = executor.clone();
                    async move {
                        let meta = executor
                            .fetch_meta_detail(&json!({"contentType": kind, "id": id}))
                            .await
                            .ok()?;
                        let videos: Vec<Value> = meta
                            .get("videos")?
                            .as_array()?
                            .iter()
                            .map(|video| {
                                json!({
                                    "id": video.get("id"),
                                    "season": video.get("season"),
                                    "episode": video.get("episode").or_else(|| video.get("number")),
                                    "released": video.get("released"),
                                })
                            })
                            .collect();
                        Some((id, videos))
                    }
                }))
                .buffer_unordered(META_FETCH_CONCURRENCY)
                .filter_map(|entry| async move { entry })
                .collect()
                .await;
                if fetched.is_empty() {
                    return;
                }
                let mut stored = executor
                    .storage
                    .read_json(&key)
                    .ok()
                    .flatten()
                    .filter(Value::is_object)
                    .unwrap_or_else(|| json!({}));
                for (id, videos) in fetched {
                    stored[id] =
                        json!({"at": chrono::Utc::now().timestamp_millis(), "videos": videos});
                }
                let _ = executor.storage.write_json(&key, &stored);
            };
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(task);
            #[cfg(not(target_arch = "wasm32"))]
            super::runtime().spawn(task);
        }
        for item in list.iter_mut() {
            if item.get("upcoming").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let Some(videos) = item
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| cache.get(id))
                .and_then(|entry| entry.get("videos"))
                .cloned()
            else {
                continue;
            };
            let Some(status) = core_value(
                "continueWatchingEpisodeStatus",
                json!({"item": item, "videos": videos, "nowMs": now_ms}),
            ) else {
                continue;
            };
            if let (Some(object), Some(status)) = (item.as_object_mut(), status.as_object()) {
                for field in ["newEpisodes", "upcoming", "airsAt"] {
                    if let Some(value) = status.get(field) {
                        object.insert(field.to_owned(), value.clone());
                    }
                }
            }
        }
        items
    }

    fn with_saved_streams(&self, profile_id: &str, mut items: Value) -> Value {
        let Some(list) = items.as_array_mut() else {
            return items;
        };
        let local = self
            .storage
            .read_json(&Storage::library_key(profile_id))
            .ok()
            .flatten()
            .and_then(|library| library.get("progress").cloned())
            .unwrap_or(Value::Null);
        let Some(local) = local.as_object() else {
            return items;
        };
        let text = |value: &Value, key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
                .map(ToOwned::to_owned)
        };
        for item in list.iter_mut() {
            let id = text(item, "id");
            let video = text(item, "lastVideoId");
            let saved = local.values().find(|entry| {
                let entry_video = text(entry, "lastVideoId");
                entry.pointer("/meta/id").and_then(Value::as_str) == id.as_deref()
                    && (video.is_none() || entry_video.is_none() || entry_video == video)
            });
            let (Some(saved), Some(fields)) = (saved, item.as_object_mut()) else {
                continue;
            };
            for key in [
                "lastStreamUrl",
                "lastStreamTitle",
                "lastStream",
                "lastStreamIndex",
                "savedAt",
            ] {
                if let Some(value) = saved.get(key).filter(|value| !value.is_null()) {
                    fields.insert(key.to_owned(), value.clone());
                }
            }
        }
        items
    }

    async fn raw_continue_watching(
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
                let items = snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([]));
                return Ok(self.with_saved_streams(profile_id, items));
            }
            Some(other) if providers::is_provider(other) => {
                let snapshot = self
                    .read_provider_library(other, profile_id, profile)
                    .await
                    .inspect_err(|error| {
                        crate::log!("[fluxa-native] {other} continue watching failed: {error}")
                    })
                    .ok();
                let items = snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([]));
                return Ok(self.with_saved_streams(profile_id, items));
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
                .ok_or_else(|| "Something went wrong. Please try again.".to_owned());
        }
        let stored_library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .unwrap_or_else(|| json!({}));
        let should_derive_continue_watching = stored_library
            .get("continueWatching")
            .is_none_or(Value::is_null);
        let mut library = core_value("normalizeLibraryDocument", stored_library)
            .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        if should_derive_continue_watching {
            let progress = library
                .get("progress")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let continue_watching = core_value("buildContinueWatchingFromProgress", progress)
                .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
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
        let nuvio = source == "nuvio";
        if provider.is_none() && !nuvio && !matches!(source.as_str(), "" | "local" | "fluxa") {
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
            None if nuvio => self
                .read_nuvio_library(profile_id, Some(&profile))
                .await?
                .unwrap_or_else(|| json!({})),
            Some(provider) => match self.cached_provider_library(provider, profile_id) {
                Some(library) => library,
                None => {
                    self.read_provider_library(provider, profile_id, &profile)
                        .await?
                }
            },
            None => self.storage.read_json(&key)?.unwrap_or_else(|| json!({})),
        };
        let library = core_value("normalizeLibraryDocument", library)
            .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let plan = core_value(
            "libraryCommandPlan",
            json!({
                "library": library,
                "command": command,
                "nowIso": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            }),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let updated = plan
            .get("library")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        match provider {
            None if nuvio => {
                self.push_nuvio_write(&profile, plan.get("externalAction").unwrap_or(&Value::Null))
                    .await?
            }
            Some(provider) => {
                let mut remote = command.clone();
                if let Some(id) = command.pointer("/item/id").and_then(Value::as_str) {
                    let liked = command.get("list").and_then(Value::as_str) == Some("liked");
                    remote["remove"] = json!(if liked {
                        !in_list(&updated, "liked", id)
                    } else {
                        !in_watchlist(&updated, id)
                    });
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
                "liked": updated.get("liked").cloned().unwrap_or_else(|| json!([])),
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

    pub(super) async fn write_playback_progress(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let profile = payload
            .get("profile")
            .filter(|value| value.is_object())
            .cloned()
            .unwrap_or_else(|| self.stored_profile(profile_id));
        let key = Storage::library_key(profile_id);
        let library = self.storage.read_json(&key)?.unwrap_or_else(|| json!({}));
        let plan = core_value(
            "playbackProgressWritePlan",
            json!({
                "library": library,
                "progress": payload.get("progress"),
                "nowIso": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "nowMs": chrono::Utc::now().timestamp_millis(),
            }),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        if let Some(external) = plan
            .get("externalProgress")
            .filter(|value| value.is_object())
            && self.nuvio_session(&profile).await?.is_some()
        {
            let mut action = external.clone();
            action["kind"] = json!("progress");
            self.push_nuvio_write(&profile, &action).await?;
            self.expire_nuvio_snapshot(&profile, "progress");
        }
        if let Some(updated) = plan.get("library").filter(|value| value.is_object()) {
            self.storage.write_json(&key, updated)?;
        }
        Ok(plan.get("entry").cloned().unwrap_or(Value::Null))
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
        let watched_series = ids.iter().find(|id| {
            local_watched_video_ids.iter().any(|watched| {
                watched
                    .as_str()
                    .is_some_and(|watched| watched.starts_with(&format!("{id}:")))
            })
        });
        let trakt_seasons = match (profile, watched_series) {
            (Some(profile), Some(series_id)) => {
                self.trakt_seasons_for_detail(profile, series_id).await
            }
            _ => None,
        };
        let addons = self.account_addons().await?;
        let provider_availability = core_value(
            "providerAvailabilityPlan",
            json!({"addons": addons.clone(), "pluginNames": []}),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let feedback = library
            .get("feedback")
            .and_then(Value::as_object)
            .and_then(|values| ids.iter().find_map(|id| values.get(*id)))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(json!({
            "savedPlayback": progress.map(progress_meta).unwrap_or(Value::Null),
            "localWatchedVideoIds": local_watched_video_ids,
            "traktSeasons": trakt_seasons,
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
        Ok(self
            .current_profile_id()?
            .unwrap_or_else(|| "guest".to_owned()))
    }

    pub(super) async fn read_nuvio_library(
        &self,
        profile_id: &str,
        effect_profile: Option<&Value>,
    ) -> Result<Option<Value>, String> {
        let mut recent = recent_library().lock().await;
        if let Some((id, at, library)) = recent.as_ref()
            && id == profile_id
            && at.elapsed() < SHARED_LIBRARY_READ
        {
            return Ok(Some(library.clone()));
        }
        let library = self
            .read_nuvio_library_uncached(profile_id, effect_profile)
            .await;
        if let Ok(Some(library)) = &library {
            *recent = Some((
                profile_id.to_owned(),
                web_time::Instant::now(),
                library.clone(),
            ));
        }
        library
    }

    async fn read_nuvio_library_uncached(
        &self,
        profile_id: &str,
        effect_profile: Option<&Value>,
    ) -> Result<Option<Value>, String> {
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
        let session = self.nuvio_session(profile).await?.ok_or_else(|| {
            "Nuvio library is selected, but this profile has no Nuvio session".to_owned()
        })?;
        self.refresh_nuvio_profiles(&session, profile).await;
        let (library, progress, history) = futures::try_join!(
            self.nuvio_synced(&session, profile, "library"),
            self.nuvio_synced(&session, profile, "progress"),
            self.nuvio_synced(&session, profile, "history"),
        )?;
        let library = library.get("items").cloned().unwrap_or_else(|| json!([]));
        let progress = progress.get("items").cloned().unwrap_or_else(|| json!([]));
        let watched = history.get("items").cloned().unwrap_or_else(|| json!([]));
        crate::log!(
            "[fluxa-native] Nuvio library sync: library_rows={} progress_rows={}",
            library.as_array().map_or(0, Vec::len),
            progress.as_array().map_or(0, Vec::len)
        );
        let metas = self.nuvio_progress_metas(&library, &progress).await;
        let metas = used_metas(metas, [&library, &progress]);
        core_value(
            "nuvioProviderLibrarySnapshot",
            json!({"library": library, "progress": progress, "watched": watched, "metas": metas}),
        )
        .map(Some)
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())
    }
}

impl EffectExecutor {
    async fn nuvio_progress_metas(&self, library: &Value, progress: &Value) -> Value {
        let mut cache = self
            .storage
            .read_json("nuvio_progress_metas")
            .ok()
            .flatten()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        if slim_cache(&mut cache) {
            let _ = self.storage.write_json("nuvio_progress_metas", &cache);
        }
        let needs = core_value(
            "nuvioProgressMetaNeeds",
            json!({"watchProgress": progress, "library": library}),
        )
        .and_then(|needs| needs.as_array().cloned())
        .unwrap_or_default();
        let progress_ids: std::collections::HashSet<&str> = progress
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.get("content_id")?.as_str())
            .collect();
        let missing = needs
            .iter()
            .filter_map(|need| {
                let id = need.get("contentId")?.as_str()?;
                let content_type = need.get("contentType")?.as_str()?;
                cache
                    .get(id)
                    .is_none()
                    .then(|| (id.to_owned(), content_type.to_owned()))
            })
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return cache;
        }
        let (mut urgent, mut deferred): (Vec<_>, Vec<_>) = missing
            .into_iter()
            .partition(|(id, _)| progress_ids.contains(id.as_str()));
        let watched_at = |id: &str| {
            progress
                .as_array()
                .into_iter()
                .flatten()
                .filter(|entry| entry.get("content_id").and_then(Value::as_str) == Some(id))
                .filter_map(|entry| entry.get("last_watched")?.as_i64())
                .max()
                .unwrap_or_default()
        };
        urgent.sort_by_key(|(id, _)| std::cmp::Reverse(watched_at(id)));
        if urgent.len() > BLOCKING_PROGRESS_METAS {
            deferred.extend(urgent.split_off(BLOCKING_PROGRESS_METAS));
        }
        let fetched = self.fetch_metas(&urgent).await;
        if !fetched.is_empty() {
            for (id, meta) in fetched {
                cache[id] = meta;
            }
            let _ = self.storage.write_json("nuvio_progress_metas", &cache);
        }
        if !deferred.is_empty() {
            let executor = self.clone();
            let task = async move {
                let fetched = executor.fetch_metas(&deferred).await;
                if fetched.is_empty() {
                    return;
                }
                let mut stored = executor
                    .storage
                    .read_json("nuvio_progress_metas")
                    .ok()
                    .flatten()
                    .filter(Value::is_object)
                    .unwrap_or_else(|| json!({}));
                for (id, meta) in fetched {
                    stored[id] = meta;
                }
                let _ = executor.storage.write_json("nuvio_progress_metas", &stored);
            };
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(task);
            #[cfg(not(target_arch = "wasm32"))]
            super::runtime().spawn(task);
        }
        cache
    }

    async fn fetch_metas(&self, wanted: &[(String, String)]) -> Vec<(String, Value)> {
        if wanted.is_empty() {
            return Vec::new();
        }
        let addons = std::sync::Arc::new(self.account_addons().await.unwrap_or_else(|_| json!([])));
        futures::stream::iter(wanted.to_vec().into_iter().map(|(id, content_type)| {
            let executor = self.clone();
            let addons = addons.clone();
            async move {
                let meta = executor
                    .fetch_meta_detail(
                        &json!({"id": id, "contentType": content_type, "addons": *addons}),
                    )
                    .await
                    .ok()?;
                meta.is_object().then(|| (id, slim_meta(&meta)))
            }
        }))
        .buffer_unordered(META_FETCH_CONCURRENCY)
        .filter_map(|entry| async move { entry })
        .collect()
        .await
    }
}

pub(super) fn in_list(library: &Value, list: &str, id: &str) -> bool {
    library
        .get(list)
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
        })
}

pub(super) fn in_watchlist(library: &Value, id: &str) -> bool {
    in_list(library, "watchlist", id)
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

fn used_metas(metas: Value, sources: [&Value; 2]) -> Value {
    let wanted: std::collections::HashSet<&str> = sources
        .into_iter()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|entry| entry.get("content_id")?.as_str())
        .collect();
    match metas {
        Value::Object(all) => Value::Object(
            all.into_iter()
                .filter(|(id, _)| wanted.contains(id.as_str()))
                .collect(),
        ),
        other => other,
    }
}

const META_FIELDS: [&str; 8] = [
    "name",
    "poster",
    "background",
    "description",
    "releaseInfo",
    "genres",
    "logo",
    "imdbRating",
];
const VIDEO_FIELDS: [&str; 7] = [
    "id",
    "season",
    "episode",
    "number",
    "name",
    "title",
    "thumbnail",
];

fn pick(source: &Value, fields: &[&str]) -> Value {
    Value::Object(
        fields
            .iter()
            .filter_map(|field| Some((field.to_string(), source.get(*field)?.clone())))
            .collect(),
    )
}

fn slim_meta(meta: &Value) -> Value {
    let mut slim = pick(meta, &META_FIELDS);
    if let Some(videos) = meta.get("videos").and_then(Value::as_array) {
        slim["videos"] = Value::Array(
            videos
                .iter()
                .map(|video| pick(video, &VIDEO_FIELDS))
                .collect(),
        );
    }
    slim
}

fn slim_cache(cache: &mut Value) -> bool {
    let Some(entries) = cache.as_object_mut() else {
        return false;
    };
    let mut changed = false;
    for meta in entries.values_mut() {
        let slim = slim_meta(meta);
        if slim != *meta {
            *meta = slim;
            changed = true;
        }
    }
    changed
}

type RecentLibrary = futures::lock::Mutex<Option<(String, web_time::Instant, Value)>>;

pub(super) fn recent_library() -> &'static RecentLibrary {
    static RECENT: std::sync::OnceLock<RecentLibrary> = std::sync::OnceLock::new();
    RECENT.get_or_init(|| futures::lock::Mutex::new(None))
}
