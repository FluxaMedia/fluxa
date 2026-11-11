use super::*;

impl EffectExecutor {
    pub fn fetch_addon_subtitles(
        &self,
        content_type: String,
        id: String,
    ) -> std::sync::mpsc::Receiver<Option<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let executor = self.clone();
        let task = async move {
            let _ = sender.send(executor.addon_subtitles(&content_type, &id).await);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }

    async fn addon_subtitles(&self, content_type: &str, id: &str) -> Option<Value> {
        let addons = self.account_addons().await.ok()?;
        let plan = core_value(
            "resourceFetchPlan",
            json!({
                "kind": "subtitles",
                "resource": "subtitles",
                "contentType": content_type,
                "id": id,
                "addons": addons
            }),
        )?;
        let requests = plan.get("requests")?.as_array()?.clone();
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(10))
            .build()
            .ok()?;
        let mut subtitles = Vec::new();
        for request in requests {
            let Some(url) = request.get("url").and_then(Value::as_str) else {
                continue;
            };
            let Ok((status_code, body)) = fetch_text(&client, url).await else {
                continue;
            };
            let parsed = core_value(
                "parseAndPlanAddonResource",
                json!({
                    "resource": "subtitles",
                    "url": url,
                    "statusCode": status_code,
                    "body": body,
                    "kind": "subtitles",
                    "addonName": request.get("addonName"),
                    "season": null
                }),
            );
            let Some(parsed) =
                parsed.filter(|p| p.get("kind").and_then(Value::as_str) == Some("success"))
            else {
                continue;
            };
            if let Some(Value::Array(found)) = parsed
                .get("valueJson")
                .and_then(Value::as_str)
                .and_then(|json| serde_json::from_str(json).ok())
            {
                subtitles.extend(found.into_iter().map(|mut subtitle| {
                    subtitle["addonName"] = request["addonName"].clone();
                    subtitle
                }));
            }
        }
        core_value("subtitleTracks", json!({ "subtitles": subtitles }))
    }
}

const META_TTL_MS: u128 = 5 * 60 * 1000;
const META_CAPACITY: usize = 64;

#[derive(Default)]
pub struct MetaCache {
    entries: std::collections::HashMap<String, (u128, Value)>,
}

impl MetaCache {
    fn get(&self, key: &str) -> Option<Value> {
        let (at, meta) = self.entries.get(key)?;
        (super::streams::now_ms().saturating_sub(*at) < META_TTL_MS).then(|| meta.clone())
    }

    fn insert(&mut self, key: String, meta: Value) {
        let now = super::streams::now_ms();
        self.entries
            .retain(|_, (at, _)| now.saturating_sub(*at) < META_TTL_MS);
        if self.entries.len() >= META_CAPACITY
            && let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, (at, _))| *at)
                .map(|(key, _)| key.clone())
        {
            self.entries.remove(&oldest);
        }
        self.entries.insert(key, (now, meta));
    }
}

fn meta_key(payload: &Value) -> String {
    let field = |name: &str| {
        payload
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
    };
    format!(
        "{}|{}|{}",
        field("contentType"),
        field("id"),
        field("sourceAddonTransportUrl")
    )
}

impl EffectExecutor {
    pub fn prefetch_meta(&self, content_type: String, id: String) {
        let executor = self.clone();
        let task = async move {
            let payload = json!({"contentType": content_type, "id": id});
            let _ = executor.fetch_meta_detail(&payload).await;
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
    }

    pub(super) async fn fetch_meta_detail(&self, payload: &Value) -> Result<Value, String> {
        let key = meta_key(payload);
        let cached = self
            .metas
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&key);
        if let Some(meta) = cached {
            return Ok(meta);
        }
        let meta = self.fetch_meta_detail_remote(payload).await?;
        if !meta.is_null() {
            self.metas
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(key, meta.clone());
        }
        Ok(meta)
    }

    async fn fetch_meta_detail_remote(&self, payload: &Value) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .ok_or_else(|| "meta detail request is missing contentType".to_owned())?;
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "meta detail request is missing id".to_owned())?;
        let addons = match payload.get("addons") {
            Some(addons) => addons.clone(),
            None => self.account_addons().await?,
        };
        let plan = core_value(
            "resourceFetchPlan",
            json!({
                "kind": "metaDetail",
                "resource": "meta",
                "contentType": content_type,
                "id": id,
                "transportUrl": payload.get("sourceAddonTransportUrl"),
                "addons": addons
            }),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let requests = plan
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for request in requests {
            let Some(url) = request.get("url").and_then(Value::as_str) else {
                continue;
            };
            let (status_code, body) = match fetch_text(&client, url).await {
                Ok(response) => response,
                Err(error) => {
                    crate::log!(
                        "[fluxa-native] meta request failed for {}: {error}",
                        url_without_query(url)
                    );
                    continue;
                }
            };
            let parsed = core_value(
                "parseAndPlanAddonResource",
                json!({
                    "resource": "meta",
                    "url": url,
                    "statusCode": status_code,
                    "body": body,
                    "kind": "metaDetail",
                    "addonName": request.get("addonName"),
                    "season": null
                }),
            );
            if parsed
                .as_ref()
                .and_then(|value| value.get("kind"))
                .and_then(Value::as_str)
                == Some("success")
            {
                return Ok(parsed
                    .and_then(|value| value.pointer("/value/meta").cloned())
                    .unwrap_or(Value::Null));
            }
        }
        Ok(Value::Null)
    }

    pub(super) async fn fetch_addon_manifest(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| "manifest request is missing transportUrl".to_owned())?;
        let plan = core_value("manifestFetchPlan", json!({"url": transport_url}))
            .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let candidates = plan
            .get("candidateUrls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for candidate in candidates.iter().filter_map(Value::as_str) {
            let Ok((status_code, Some(body))) = fetch_text(&client, candidate).await else {
                continue;
            };
            if !(200..300).contains(&status_code) {
                continue;
            }
            let host = reqwest::Url::parse(candidate)
                .ok()
                .and_then(|url| url.host_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "Unknown Addon".to_owned());
            let Some(descriptor) = core_value(
                "parseManifest",
                json!({
                    "body": body,
                    "transportUrl": plan.get("normalizedTransportUrl").and_then(Value::as_str).unwrap_or(transport_url),
                    "unknownName": host,
                }),
            ) else {
                continue;
            };
            return core_value("resolveManifestAssets", descriptor.clone())
                .or(Some(descriptor))
                .ok_or_else(|| "Something went wrong. Please try again.".to_owned());
        }
        Err("addon manifest could not be fetched or parsed".to_owned())
    }

    pub(super) async fn fetch_addon_resource(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "addon resource is missing transportUrl".to_owned())?;
        let resource = payload
            .get("resource")
            .and_then(Value::as_str)
            .unwrap_or("catalog");
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let extra = payload.get("extra").cloned().unwrap_or(Value::Null);
        let extra_args = extra
            .as_object()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|(key, value)| {
                        let text = value
                            .as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| value.as_i64().map(|number| number.to_string()))
                            .or_else(|| value.as_bool().map(|boolean| boolean.to_string()))?;
                        Some((key.clone(), Value::String(text)))
                    })
                    .collect::<serde_json::Map<_, _>>()
            })
            .unwrap_or_default();
        let extra_raw = payload
            .get("extraArgs")
            .and_then(Value::as_str)
            .or_else(|| extra.as_str())
            .unwrap_or_default();
        let request_plan = core_value(
            "addonResourceRequestPlan",
            json!({
                "transportUrl": transport_url,
                "resource": resource,
                "contentType": content_type,
                "id": id,
                "extraArgs": extra_args,
                "extraRaw": extra_raw,
            }),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?;
        let urls = request_plan
            .get("urls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let kind = match resource {
            "catalog" | "metas" => "catalogPage",
            "meta" => "metaDetail",
            "stream" | "streams" => "streams",
            "subtitle" | "subtitles" => "subtitles",
            other => other,
        };
        let addon_name = payload.get("addonName").and_then(Value::as_str);
        let season = payload.get("season").and_then(Value::as_i64);
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for url in urls.iter().filter_map(Value::as_str) {
            let Ok((status_code, body)) = fetch_text(&client, url).await else {
                continue;
            };
            let Some(parsed) = core_value(
                "parseAndPlanAddonResource",
                json!({
                    "resource": resource,
                    "url": url,
                    "statusCode": status_code,
                    "body": body,
                    "kind": kind,
                    "addonName": addon_name,
                    "season": season,
                }),
            ) else {
                continue;
            };
            if parsed.get("kind").and_then(Value::as_str) == Some("success") {
                if let Some(value_json) = parsed.get("valueJson").and_then(Value::as_str) {
                    if let Ok(value) = serde_json::from_str(value_json) {
                        return Ok(value);
                    }
                }
            }
        }
        Ok(if kind == "metaDetail" {
            Value::Null
        } else {
            Value::Array(Vec::new())
        })
    }
}

pub(super) fn normalize_addons(addons: Value, keep_disabled: bool) -> Result<Value, String> {
    let mut addons = if keep_disabled {
        addons
    } else {
        core_value(
            "filterEnabledAddons",
            json!({"addons": addons, "disabledKeys": []}),
        )
        .ok_or_else(|| "Something went wrong. Please try again.".to_owned())?
    };
    if let Some(items) = addons.as_array_mut() {
        for item in items {
            if item
                .get("manifest")
                .and_then(|manifest| manifest.get("id"))
                .and_then(Value::as_str)
                .is_some()
            {
                continue;
            }
            if let Some(normalized) = core_value(
                "normalizeAddonDescriptor",
                json!({"addonJson": item.to_string()}),
            ) {
                *item = normalized;
            }
        }
    }
    Ok(addons)
}

#[cfg(test)]
mod meta_cache_tests {
    use super::*;

    #[test]
    fn repeat_open_of_the_same_title_reuses_the_fetched_meta() {
        let mut cache = MetaCache::default();
        let payload = json!({"contentType": "series", "id": "tt1"});
        assert!(cache.get(&meta_key(&payload)).is_none());
        cache.insert(meta_key(&payload), json!({"name": "Show"}));
        assert_eq!(
            cache.get(&meta_key(&payload)),
            Some(json!({"name": "Show"}))
        );
        let other = json!({"contentType": "movie", "id": "tt1"});
        assert!(cache.get(&meta_key(&other)).is_none());
    }

    #[test]
    fn expired_meta_is_refetched() {
        let mut cache = MetaCache::default();
        cache
            .entries
            .insert("series|tt1|".to_owned(), (0, json!({"name": "Old"})));
        assert!(cache.get("series|tt1|").is_none());
    }

    #[test]
    fn oldest_entry_goes_first_when_the_cache_is_full() {
        let mut cache = MetaCache::default();
        let now = super::super::streams::now_ms();
        for index in 0..META_CAPACITY {
            cache.entries.insert(
                format!("movie|tt{index}|"),
                (now - index as u128, Value::Null),
            );
        }
        cache.insert("movie|new|".to_owned(), json!({}));
        assert!(cache.entries.contains_key("movie|new|"));
        assert!(
            !cache
                .entries
                .contains_key(&format!("movie|tt{}|", META_CAPACITY - 1))
        );
        assert_eq!(cache.entries.len(), META_CAPACITY);
    }
}
