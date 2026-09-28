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
        let mut addons = self
            .addons_for_profile(&profiles, &active_id, &profile)
            .await?;
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

        let mut collection_profile = profile.clone();
        if let Ok(Some(session)) = self.nuvio_session(&profile).await
            && let Ok(rows) = self
                .nuvio_rows(
                    &session,
                    &profile,
                    "collections",
                    "sync_pull_collections",
                    json!({"p_profile_id": session.profile_index}),
                )
                .await
            && let Some(collections) = rows
                .as_array()
                .and_then(|rows| rows.first())
                .and_then(|row| row.get("collections_json"))
                .and_then(Value::as_array)
            && let Some(mapped) = core_value(
                "nuvioMapCollections",
                json!({"collections": collections, "profileIndex": session.profile_index}),
            )
            && let Some(fields) = collection_profile.as_object_mut()
        {
            fields.insert("libraryCollections".to_owned(), mapped);
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
        if let Ok(Some(session)) = self.nuvio_session(&profile).await
            && let Some(items) = self.nuvio_home_catalog_items(&session, &profile).await
            && let Some(Value::Array(ordered)) = core_value(
                "nuvioHomeLayout",
                json!({"categories": all_categories, "addons": addons, "items": items}),
            )
        {
            all_categories = ordered;
        }
        let continue_watching = self
            .continue_watching_for_source(&active_id, &profile, &prefs, None)
            .await?;
        let billboard = all_categories
            .iter()
            .filter(|category| category.get("type").and_then(Value::as_str) != Some("collection"))
            .find_map(|category| {
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
}
