use super::*;

impl EffectExecutor {
    pub(super) async fn read_home_bootstrap(&self, payload: &Value) -> Result<Value, String> {
        let language = payload
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or("en");
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        let cache_key = format!(
            "home_bootstrap_v1_{}_{}",
            Storage::library_key(profile_id),
            sanitize_key(language)
        );
        if payload.get("force").and_then(Value::as_bool) != Some(true) {
            if let Some(Value::Object(mut cached)) = self.storage.read_json(&cache_key)? {
                cached.insert("stale".to_owned(), Value::Bool(true));
                return Ok(Value::Object(cached));
            }
        }

        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| profile_id.to_owned());
        let profile = profiles
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(active_id.as_str()))
            })
            .cloned()
            .unwrap_or_else(|| json!({}));
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(&active_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let profiles_request = json!({"profiles": profiles, "activeProfileId": active_id});
        let owner = core_value("effectiveAddonsOwnerId", profiles_request)
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| active_id.clone());
        let mut addons = self
            .storage
            .read_json(&Storage::addons_key(&owner))?
            .or_else(|| {
                self.storage
                    .read_json(&Storage::addons_key(&active_id))
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| json!([]));
        if !addons.is_array() {
            addons = json!([]);
        }
        addons = normalize_enabled_addons(addons)?;
        if let Some(api_key) = prefs
            .get("tmdbApiKey")
            .and_then(Value::as_str)
            .filter(|key| !key.trim().is_empty())
        {
            if let Some(manifest) = core_value("tmdbBuiltinManifest", json!({})).and_then(|value| {
                value
                    .as_str()
                    .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            }) {
                if let Some(items) = addons.as_array_mut() {
                    let descriptor = json!({"transportUrl": "tmdb://builtin", "manifest": manifest, "apiKey": api_key});
                    if prefs.get("tmdbPreferOverAddons").and_then(Value::as_bool) == Some(true) {
                        items.insert(0, descriptor);
                    } else {
                        items.push(descriptor);
                    }
                }
            }
        }

        // Discover already resolves add-ons from the signed-in Nuvio account,
        // but Home used only the local add-on cache. Consequently the user's
        // collection shelves loaded while their catalog shelves (and catalog-
        // sourced hero candidates) silently vanished. Merge the same remote
        // manifests into this bootstrap before deriving metadata feeds.
        self.merge_nuvio_addons(&profile, &mut addons).await;

        let addon_json = addons.to_string();
        let feeds =
            core_value("buildMetadataFeedOptions", addons.clone()).unwrap_or_else(|| json!([]));
        let feed_count = feeds.as_array().map_or(0, Vec::len);
        let visible_feeds = core_value(
            "homeMetadataFeedPlan",
            json!({
                "feeds": feeds,
                "selectedKeys": prefs.get("homeFeedToggles").unwrap_or(&Value::Null),
                "order": prefs.get("homeFeedOrder").unwrap_or(&Value::Null),
            }),
        )
        .and_then(|value| value.as_array().cloned())
        .ok_or_else(|| "Fluxa Core could not resolve selected home metadata feeds".to_owned())?;
        let metadata_feeds = visible_feeds.clone();
        crate::log!(
            "[fluxa-native] home bootstrap: addons={} feeds={} visible_feeds={}",
            addons.as_array().map_or(0, Vec::len),
            feed_count,
            visible_feeds.len()
        );

        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let mut categories = Vec::new();
        for feed in visible_feeds {
            let content_type = feed.get("type").and_then(Value::as_str).unwrap_or("movie");
            let catalog_id = feed.get("id").and_then(Value::as_str).unwrap_or_default();
            let transport_url = feed
                .get("transportUrl")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let extra = json!({});
            let data = if transport_url == "tmdb://builtin" {
                let Some(api_key) = prefs.get("tmdbApiKey").and_then(Value::as_str) else {
                    continue;
                };
                let url = core_value(
                    "tmdbBuiltinCatalogUrl",
                    json!({"contentType": content_type, "extra": extra, "apiKey": api_key, "language": language}),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned));
                let Some(url) = url else { continue };
                let response = match fetch_json(&client, &url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] TMDB catalog request failed for {}: {error}",
                            url_without_query(&url)
                        );
                        continue;
                    }
                };
                let results = response
                    .get("results")
                    .cloned()
                    .unwrap_or_else(|| json!([]));
                core_value(
                    "tmdbBulkMetas",
                    json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
                )
                .unwrap_or_else(|| json!([]))
            } else {
                let Some(url) = core_value(
                    "buildResourceUrl",
                    json!({
                        "transportUrl": transport_url,
                        "resource": "catalog",
                        "contentType": content_type,
                        "id": catalog_id,
                        "extraJson": extra.to_string()
                    }),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned)) else {
                    continue;
                };
                let (status_code, body) = match fetch_text(&client, &url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] catalog request failed for {}: {error}",
                            url_without_query(&url)
                        );
                        continue;
                    }
                };
                match parse_catalog_items(&url, status_code, body.as_deref(), "discover") {
                    Ok(items) => Value::Array(items),
                    Err(error) => {
                        crate::log!("[fluxa-native] catalog response rejected by Core: {error}");
                        continue;
                    }
                }
            };
            let items = annotate_catalog_items(
                data.as_array().map(Vec::as_slice).unwrap_or_default(),
                transport_url,
                content_type,
            )?;
            if items.is_empty() {
                continue;
            }
            let label = feed
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("Catalog");
            let home_title = feed
                .get("homeTitle")
                .and_then(Value::as_str)
                .unwrap_or(label);
            categories.push(json!({
                "id": feed.get("key"),
                "name": home_title,
                "semanticName": home_title,
                "type": content_type,
                "items": items,
                "addonName": label.split(" - ").next().unwrap_or(label),
                "transportUrl": transport_url,
                "catalogId": catalog_id,
                "canLoadMore": true
            }));
        }
        crate::log!(
            "[fluxa-native] home bootstrap: populated_catalog_shelves={}",
            categories.len()
        );

        // The Web home pipeline hydrates Nuvio's cloud collections before
        // asking Core to build collection shelves. Do the same here instead
        // of silently rendering the profile's stale local copy.
        let mut collection_profile = profile.clone();
        if let (Some(base_url), Some(api_key), Some(token)) = (
            option_env!("FLUXA_NUVIO_SUPABASE_URL"),
            option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty()),
        ) {
            let profile_index = profile
                .get("nuvioProfileIndex")
                .and_then(Value::as_i64)
                .filter(|index| *index > 0)
                .unwrap_or(1);
            let endpoint = format!(
                "{}/rest/v1/rpc/sync_pull_collections",
                base_url.trim_end_matches('/')
            );
            let remote = client
                .post(endpoint)
                .header("apikey", api_key)
                .bearer_auth(token)
                .json(&json!({"p_profile_id": profile_index}))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status);
            if let Ok(response) = remote
                && let Ok(rows) = response.json::<Value>().await
                && let Some(collections) = rows
                    .as_array()
                    .and_then(|rows| rows.first())
                    .and_then(|row| row.get("collections_json"))
                    .and_then(Value::as_array)
                && let Some(mapped) = core_value(
                    "nuvioMapCollections",
                    json!({"collections": collections, "profileIndex": profile_index}),
                )
                && let Some(fields) = collection_profile.as_object_mut()
            {
                fields.insert("libraryCollections".to_owned(), mapped);
            }
        }
        let shelves = core_value(
            "buildHomeCollectionShelves",
            json!({"profileJson": collection_profile.to_string(), "addonsJson": addon_json}),
        )
        .unwrap_or_else(
            || json!({"pinnedShelves": [], "regularShelves": [], "hiddenFolderCategories": []}),
        );
        let mut all_categories = shelves
            .get("pinnedShelves")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        all_categories.extend(categories);
        all_categories.extend(
            shelves
                .get("regularShelves")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
        all_categories.extend(
            shelves
                .get("hiddenFolderCategories")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
        let continue_watching = self
            .continue_watching_for_source(&active_id, &profile, &prefs, None)
            .await?;
        let billboard = all_categories.iter().find_map(|category| {
            category
                .get("items")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .cloned()
        });
        if all_categories.is_empty() {
            if let Some(Value::Object(mut cached)) = self.storage.read_json(&cache_key)? {
                cached.insert("stale".to_owned(), Value::Bool(false));
                return Ok(Value::Object(cached));
            }
        }
        let bootstrap = json!({
            "categories": all_categories,
            "continueWatching": continue_watching,
            "metadataFeeds": metadata_feeds,
            "billboard": billboard
        });
        self.storage.write_json(&cache_key, &bootstrap)?;
        Ok(bootstrap)
    }

    pub(super) async fn nuvio_addons_profile_index(
        &self,
        client: &Client,
        base_url: &str,
        api_key: &str,
        token: &str,
        profile_index: i64,
    ) -> i64 {
        if profile_index == 1 {
            return 1;
        }
        let response = client
            .post(format!(
                "{}/rest/v1/rpc/sync_pull_profiles",
                base_url.trim_end_matches('/')
            ))
            .header("apikey", api_key)
            .bearer_auth(token)
            .json(&json!({}))
            .send()
            .await;
        let remote_profiles = match response {
            Ok(response) if response.status().is_success() => {
                match response.json::<Vec<Value>>().await {
                    Ok(profiles) => profiles,
                    Err(error) => {
                        crate::log!("[fluxa-native] Nuvio profile scopes decode failed: {error}");
                        return profile_index;
                    }
                }
            }
            Ok(response) => {
                crate::log!(
                    "[fluxa-native] Nuvio profile scopes request failed: HTTP {}",
                    response.status()
                );
                return profile_index;
            }
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio profile scopes request failed: {error}");
                return profile_index;
            }
        };
        core_value(
            "nuvioEffectiveProfileScopes",
            json!({"profileIndex": profile_index, "profiles": remote_profiles}),
        )
        .and_then(|scopes| scopes.get("addons").and_then(Value::as_i64))
        .unwrap_or(profile_index)
    }

    pub(super) async fn merge_nuvio_addons(&self, profile: &Value, addons: &mut Value) {
        let (Some(base_url), Some(api_key), Some(token)) = (
            option_env!("FLUXA_NUVIO_SUPABASE_URL"),
            option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty()),
        ) else {
            return;
        };
        let profile_index = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .filter(|index| *index > 0)
            .unwrap_or(1);
        let client = match Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(12))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio add-on client unavailable: {error}");
                return;
            }
        };
        let addon_profile_index = self
            .nuvio_addons_profile_index(&client, base_url, api_key, token, profile_index)
            .await;
        let endpoint = format!(
            "{}/rest/v1/addons?select=*&profile_id=eq.{}&order=sort_order",
            base_url.trim_end_matches('/'),
            addon_profile_index
        );
        let remote_addons = match client
            .get(endpoint)
            .header("apikey", api_key)
            .bearer_auth(token)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => match response.json::<Vec<Value>>().await {
                Ok(addons) => addons,
                Err(error) => {
                    crate::log!("[fluxa-native] Nuvio add-on list response invalid: {error}");
                    return;
                }
            },
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio add-on list request failed: {error}");
                return;
            }
        };
        let Some(items) = addons.as_array_mut() else {
            return;
        };
        for remote in remote_addons {
            if remote.get("enabled").and_then(Value::as_bool) == Some(false) {
                continue;
            }
            let Some(url) = remote.get("url").and_then(Value::as_str) else {
                continue;
            };
            if items
                .iter()
                .any(|item| item.get("transportUrl").and_then(Value::as_str) == Some(url))
            {
                continue;
            }
            let manifest = match client.get(url).send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => match response.json::<Value>().await {
                        Ok(manifest) => manifest,
                        Err(error) => {
                            crate::log!("[fluxa-native] Nuvio add-on manifest invalid: {error}");
                            continue;
                        }
                    },
                    Err(error) => {
                        crate::log!("[fluxa-native] Nuvio add-on manifest request failed: {error}");
                        continue;
                    }
                },
                Err(error) => {
                    crate::log!("[fluxa-native] Nuvio add-on manifest request failed: {error}");
                    continue;
                }
            };
            let descriptor = json!({
                "transportUrl": url,
                "manifest": manifest,
                "name": remote.get("name").cloned().unwrap_or(Value::Null),
                "enabled": true,
            });
            if let Some(descriptor) = core_value(
                "normalizeAddonDescriptor",
                json!({"addonJson": descriptor.to_string()}),
            ) {
                items.push(descriptor);
            }
        }
    }
}
