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
                subtitles.extend(found);
            }
        }
        core_value("subtitleTracks", json!({ "subtitles": subtitles }))
    }
}

impl EffectExecutor {
    pub(super) async fn fetch_meta_detail(&self, payload: &Value) -> Result<Value, String> {
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
        .ok_or_else(|| "Fluxa Core could not plan the meta detail request".to_owned())?;
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
            .ok_or_else(|| "Fluxa Core could not plan manifest requests".to_owned())?;
        let candidates = plan
            .get("candidateUrls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
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
                .ok_or_else(|| "Fluxa Core could not resolve manifest assets".to_owned());
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
        .ok_or_else(|| "Fluxa Core could not plan addon resource URLs".to_owned())?;
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

pub(super) fn normalize_enabled_addons(addons: Value) -> Result<Value, String> {
    let mut addons = core_value(
        "filterEnabledAddons",
        json!({"addons": addons, "disabledKeys": []}),
    )
    .ok_or_else(|| "Fluxa Core could not filter enabled add-ons".to_owned())?;
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
