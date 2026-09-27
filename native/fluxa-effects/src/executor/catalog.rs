use super::*;

impl EffectExecutor {
    pub(super) async fn read_discover_catalog_filters(
        &self,
        payload: &Value,
    ) -> Result<Value, String> {
        let effect_profile = payload
            .get("profile")
            .filter(|value| value.is_object())
            .cloned();
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .or_else(|| effect_profile.as_ref()?.get("id").and_then(Value::as_str))
            .unwrap_or("guest");
        let mut profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let stored_active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| profile_id.to_owned());
        let active_id = effect_profile
            .as_ref()
            .and_then(|profile| profile.get("id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(&stored_active_id)
            .to_owned();
        if let Some(profile) = effect_profile.as_ref()
            && let Some(items) = profiles.as_array_mut()
        {
            if let Some(existing) = items.iter_mut().find(|item| {
                item.get("id").and_then(Value::as_str) == profile.get("id").and_then(Value::as_str)
            }) {
                *existing = profile.clone();
            } else {
                items.push(profile.clone());
            }
        }
        let owner = core_value(
            "effectiveAddonsOwnerId",
            json!({"profiles": profiles, "activeProfileId": active_id}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| profile_id.to_owned());
        let mut addons = payload
            .get("addons")
            .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
            .cloned()
            .or(self
                .storage
                .read_json(&Storage::addons_key(&owner))?
                .or_else(|| {
                    self.storage
                        .read_json(&Storage::addons_key(profile_id))
                        .ok()
                        .flatten()
                })
                .or_else(|| self.storage.read_json("addons").ok().flatten()))
            .unwrap_or_else(|| json!([]));
        if !addons.is_array() {
            addons = json!([]);
        }
        // Home normalizes cached/legacy descriptors before deriving catalog
        // feeds. Discover must use the same Core-normalized input or installed
        // addons can appear on Home while this screen has no selectable catalogs.
        addons = normalize_enabled_addons(addons)?;
        let cached_addon_count = addons.as_array().map_or(0, Vec::len);

        // Home and Discover must use the same effective add-on inventory.
        // Refresh a stale Nuvio token here, then reuse Home's single remote
        // profile/add-on/manifest hydration path instead of maintaining a
        // second implementation that could silently produce an empty list.
        if let Some(profile) = profiles.as_array().and_then(|items| {
            items.iter().find(|item| {
                item.get("id").and_then(Value::as_str) == Some(active_id.as_str())
                    || item.get("id").and_then(Value::as_str) == Some(profile_id)
            })
        }) {
            let mut profile_for_addons = profile.clone();
            let mut session = profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            if let (Some(base_url), Some(api_key)) = (
                option_env!("FLUXA_NUVIO_SUPABASE_URL"),
                option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            ) {
                let client = Client::builder()
                    .user_agent("Fluxa/1.0")
                    .native_timeout(Duration::from_secs(12))
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
                    && let Ok(response) = client
                        .post(format!(
                            "{}/auth/v1/token?grant_type=refresh_token",
                            base_url.trim_end_matches('/')
                        ))
                        .header("apikey", api_key)
                        .json(&json!({"refresh_token": refresh_token}))
                        .send()
                        .await
                        .and_then(reqwest::Response::error_for_status)
                    && let Ok(refreshed) = response.json::<Value>().await
                    && let Some(access_token) =
                        refreshed.get("access_token").and_then(Value::as_str)
                {
                    session = Some(access_token.to_owned());
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
                    profile_for_addons = updated_profile.clone();
                    if let Some(items) = profiles.as_array() {
                        let updated = items
                            .iter()
                            .map(|item| {
                                if item.get("id").and_then(Value::as_str)
                                    == profile.get("id").and_then(Value::as_str)
                                {
                                    updated_profile.clone()
                                } else {
                                    item.clone()
                                }
                            })
                            .collect::<Vec<_>>();
                        self.storage.write_json("profiles", &json!(updated))?;
                    }
                }
                if let Some(session) = session {
                    if let Some(object) = profile_for_addons.as_object_mut() {
                        object.insert("nuvioAccessToken".into(), json!(session));
                    }
                }
            }
            self.merge_nuvio_addons(&profile_for_addons, &mut addons)
                .await;
        }
        crate::log!(
            "[fluxa-native] Discover catalogs: cached_addons={} hydrated_remote_addons={}",
            cached_addon_count,
            addons
                .as_array()
                .map_or(0, Vec::len)
                .saturating_sub(cached_addon_count)
        );
        if let Some(items) = addons.as_array_mut() {
            for item in items {
                if item
                    .get("manifest")
                    .and_then(|manifest| manifest.get("id"))
                    .and_then(Value::as_str)
                    .is_none()
                    && let Some(normalized) = core_value(
                        "normalizeAddonDescriptor",
                        json!({"addonJson": item.to_string()}),
                    )
                {
                    *item = normalized;
                }
            }
        }
        Ok(json!({"addons": addons}))
    }

    pub(super) async fn run_discover(&self, payload: &Value) -> Result<Value, String> {
        let requests = core_value("discoverSourceRequests", payload.clone())
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| "Fluxa Core could not plan Discover requests".to_owned())?;
        let mut sources = Vec::with_capacity(requests.len());
        for request in requests {
            let transport_url = request
                .get("transportUrl")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing transportUrl".to_owned())?;
            let request_type = request
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing type".to_owned())?;
            let catalog_id = request
                .get("catalogId")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing catalogId".to_owned())?;
            let extra = request.get("extra").cloned().unwrap_or_else(|| json!({}));
            // Match Android: a failing media type must not discard successful sibling results.
            let items = self
                .fetch_addon_catalog_items(
                    transport_url,
                    request_type,
                    catalog_id,
                    &extra,
                    "discover",
                )
                .await
                .unwrap_or_default();
            sources.push(json!({
                "transportUrl": transport_url,
                "catalogId": catalog_id,
                "type": request_type,
                "genre": request.get("genre"),
                "items": items
            }));
        }
        core_value("mergeDiscoverSources", json!({"sources": sources}))
            .ok_or_else(|| "Fluxa Core could not merge Discover sources".to_owned())
    }

    pub(super) async fn run_search(&self, payload: &Value) -> Result<Value, String> {
        let query = payload
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        if query.is_empty() {
            return Ok(json!({"results": [], "categories": [], "grouping": {
                "groups": [], "totalCount": 0, "query": query
            }}));
        }

        let addons = self.load_enabled_addons()?;
        let plan = core_value(
            "resourceFetchPlan",
            json!({"kind": "search", "query": query, "addons": addons}),
        )
        .ok_or_else(|| "Fluxa Core could not plan search resources".to_owned())?;
        let requests = plan
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let fetches = requests.iter().map(|request| {
            let client = &client;
            async move {
                let url = request.get("url").and_then(Value::as_str)?;
                let transport_url = request
                    .get("transportUrl")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let catalog_type = request
                    .get("catalogType")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let items = if transport_url.starts_with("tmdb://builtin")
                    || url.starts_with("tmdb://builtin")
                {
                    match self
                        .fetch_builtin_tmdb_items(catalog_type, &json!({"search": query}))
                        .await
                    {
                        Ok(items) => items,
                        Err(error) => {
                            crate::log!("[fluxa-native] builtin search failed: {error}");
                            return None;
                        }
                    }
                } else {
                    let (status_code, body) = match fetch_text(client, url).await {
                        Ok(response) => response,
                        Err(error) => {
                            crate::log!(
                                "[fluxa-native] search request failed for {}: {error}",
                                url_without_query(url)
                            );
                            return None;
                        }
                    };
                    let items =
                        match parse_catalog_items(url, status_code, body.as_deref(), "search") {
                            Ok(items) => items,
                            Err(error) => {
                                crate::log!(
                                    "[fluxa-native] search response rejected by Core: {error}"
                                );
                                return None;
                            }
                        };
                    annotate_catalog_items(&items, transport_url, catalog_type).ok()?
                };
                if items.is_empty() {
                    return None;
                }
                let source_name = request
                    .get("categoryName")
                    .or_else(|| request.get("addonName"))
                    .cloned()
                    .unwrap_or(Value::Null);
                Some(json!({
                    "id": request.get("categoryId").cloned().unwrap_or_else(|| json!(url)),
                    "name": source_name,
                    "semanticName": source_name,
                    "type": request.get("catalogType"),
                    "items": items,
                    "addonName": request.get("addonName"),
                    "catalogId": request.get("catalogId")
                }))
            }
        });
        let sources: Vec<Value> = futures::future::join_all(fetches)
            .await
            .into_iter()
            .flatten()
            .collect();

        let merged = core_value("mergeSearchSources", Value::Array(sources))
            .ok_or_else(|| "Fluxa Core could not merge search results".to_owned())?;
        let results = merged.get("results").cloned().unwrap_or_else(|| json!([]));
        let grouping = core_value(
            "searchResultGrouping",
            json!({"query": query, "results": results}),
        )
        .unwrap_or_else(|| json!({"groups": [], "totalCount": 0, "query": query}));
        Ok(json!({
            "results": results,
            "categories": merged.get("categories").cloned().unwrap_or_else(|| json!([])),
            "grouping": grouping
        }))
    }

    pub(super) async fn fetch_catalog_page(&self, payload: &Value) -> Result<Value, String> {
        if payload
            .get("remoteSource")
            .is_some_and(|source| !source.is_null() && source != &json!([]))
        {
            return Err("desktop remote collection paging is not implemented".to_owned());
        }
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog page is missing transportUrl".to_owned())?;
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let catalog_id = payload
            .get("catalogId")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog page is missing catalogId".to_owned())?;
        let mut extra = serde_json::Map::new();
        if let Some(skip) = payload.get("skip").and_then(Value::as_i64) {
            extra.insert("skip".to_owned(), json!(skip.max(0)));
        }
        if let Some(genre) = payload.get("genre").and_then(Value::as_str) {
            extra.insert("genre".to_owned(), json!(genre));
        }
        if let Some(search) = payload.get("search").and_then(Value::as_str) {
            if !search.trim().is_empty() {
                extra.insert("search".to_owned(), json!(search));
            }
        }
        let items = self
            .fetch_addon_catalog_items(
                transport_url,
                content_type,
                catalog_id,
                &Value::Object(extra),
                "catalogPage",
            )
            .await?;
        Ok(json!({"items": items}))
    }

    pub(super) async fn fetch_discover_page(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| "discover page is missing transportUrl".to_owned())?;
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let catalog_id = payload
            .get("catalogId")
            .and_then(Value::as_str)
            .ok_or_else(|| "discover page is missing catalogId".to_owned())?;
        let mut extra = serde_json::Map::new();
        if let Some(skip) = payload.get("skip").and_then(Value::as_i64) {
            extra.insert("skip".to_owned(), json!(skip.max(0)));
        }
        if let Some(genre) = payload.get("genre").and_then(Value::as_str) {
            extra.insert("genre".to_owned(), json!(genre));
        }
        if let Some(search) = payload
            .get("search")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        {
            extra.insert("search".to_owned(), json!(search));
        }
        let items = self
            .fetch_addon_catalog_items(
                transport_url,
                content_type,
                catalog_id,
                &Value::Object(extra),
                "catalogPage",
            )
            .await?;
        Ok(json!({"items": items}))
    }

    pub(super) async fn fetch_addon_catalog_items(
        &self,
        transport_url: &str,
        content_type: &str,
        catalog_id: &str,
        extra: &Value,
        parse_kind: &str,
    ) -> Result<Vec<Value>, String> {
        if transport_url == "tmdb://builtin" {
            return self.fetch_builtin_tmdb_items(content_type, extra).await;
        }
        let url = core_value(
            "buildResourceUrl",
            json!({
                "transportUrl": transport_url,
                "resource": "catalog",
                "contentType": content_type,
                "id": catalog_id,
                "extraJson": extra.to_string()
            }),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| "Fluxa Core could not build the catalog page URL".to_owned())?;
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let (status_code, body) = fetch_text(&client, &url).await?;
        let items = parse_catalog_items(&url, status_code, body.as_deref(), parse_kind)?;
        annotate_catalog_items(&items, transport_url, content_type)
    }

    pub(super) async fn fetch_builtin_tmdb_items(
        &self,
        content_type: &str,
        extra: &Value,
    ) -> Result<Vec<Value>, String> {
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_default();
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(&active_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let api_key = prefs
            .get("tmdbApiKey")
            .and_then(Value::as_str)
            .ok_or_else(|| "TMDB API key is not configured".to_owned())?;
        let language = self
            .storage
            .read_json("profiles")?
            .and_then(|profiles| {
                profiles.as_array()?.iter().find_map(|profile| {
                    (profile.get("id").and_then(Value::as_str) == Some(active_id.as_str()))
                        .then(|| profile.get("language")?.as_str().map(ToOwned::to_owned))
                        .flatten()
                })
            })
            .unwrap_or_else(|| "en".to_owned());
        let url = core_value(
            "tmdbBuiltinCatalogUrl",
            json!({"contentType": content_type, "extra": extra, "apiKey": api_key, "language": language}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| "Fluxa Core could not build the TMDB catalog URL".to_owned())?;
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let response = fetch_json(&client, &url).await?;
        let results = response
            .get("results")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let metas = core_value(
            "tmdbBulkMetas",
            json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
        )
        .unwrap_or_else(|| json!([]));
        annotate_catalog_items(
            metas.as_array().map(Vec::as_slice).unwrap_or_default(),
            "tmdb://builtin",
            content_type,
        )
    }
}

pub(super) fn parse_catalog_items(
    url: &str,
    status_code: i32,
    body: Option<&str>,
    kind: &str,
) -> Result<Vec<Value>, String> {
    let result = core_value(
        "parseAndPlanAddonResource",
        json!({
            "resource": "catalog",
            "url": url,
            "statusCode": status_code,
            "body": body,
            "kind": kind,
            "season": null,
        }),
    )
    .ok_or_else(|| "Core could not parse the catalog response".to_owned())?;
    match result.get("kind").and_then(Value::as_str) {
        Some("success") => {}
        Some("empty") => return Ok(Vec::new()),
        _ => {
            return Err(result
                .get("error")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "Core returned {} for {url}",
                        result
                            .get("kind")
                            .and_then(Value::as_str)
                            .unwrap_or("an invalid response")
                    )
                }));
        }
    }
    Ok(result
        .pointer("/value/items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub(super) fn annotate_catalog_items(
    items: &[Value],
    transport_url: &str,
    catalog_type: &str,
) -> Result<Vec<Value>, String> {
    core_value(
        "annotateCatalogItems",
        json!({
            "itemsJson": Value::Array(items.to_vec()).to_string(),
            "transportUrl": transport_url,
            "catalogType": catalog_type
        }),
    )
    .and_then(|value| value.as_array().cloned())
    .ok_or_else(|| "Fluxa Core could not annotate catalog source metadata".to_owned())
}
