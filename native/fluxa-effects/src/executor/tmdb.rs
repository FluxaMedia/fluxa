use super::*;

impl EffectExecutor {
    pub fn fetch_json(&self, url: String) -> std::sync::mpsc::Receiver<Option<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let response = async {
                Client::builder()
                    .native_timeout(Duration::from_secs(10))
                    .build()?
                    .get(&url)
                    .header("User-Agent", "Fluxa")
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Value>()
                    .await
            }
            .await;
            let _ = sender.send(response.ok());
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }

    pub fn fetch_trailers(
        &self,
        items: Vec<Value>,
        api_key: String,
        language: String,
    ) -> std::sync::mpsc::Receiver<Vec<(String, Value)>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let mut found = Vec::new();
            if let Ok(client) = Client::builder()
                .native_timeout(Duration::from_secs(10))
                .build()
            {
                for item in items {
                    let Some(id) = item.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    let trailers = tmdb_trailers(&client, &item, &api_key, &language).await;
                    found.push((id.to_owned(), trailers.unwrap_or_else(|| json!([]))));
                }
            }
            let _ = sender.send(found);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }

    pub fn fetch_similar(
        &self,
        item: Value,
        api_key: String,
        language: String,
        endpoints: Vec<&'static str>,
    ) -> std::sync::mpsc::Receiver<Vec<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let items = match Client::builder()
                .native_timeout(Duration::from_secs(10))
                .build()
            {
                Ok(client) => tmdb_similar(&client, &item, &api_key, &language, &endpoints)
                    .await
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };
            let _ = sender.send(items);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }

    pub fn fetch_trending(&self, api_key: String) -> std::sync::mpsc::Receiver<Vec<Value>> {
        detached(async move {
            let Ok(client) = Client::builder()
                .native_timeout(Duration::from_secs(10))
                .build()
            else {
                return Vec::new();
            };
            let mut results = Vec::new();
            for page in 1..=2 {
                let url = format!(
                    "https://api.themoviedb.org/3/trending/all/week?api_key={api_key}&page={page}"
                );
                let Ok(body) = fetch_json(&client, &url).await else {
                    break;
                };
                results.extend(
                    body.get("results")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                );
            }
            results
        })
    }

    pub fn fetch_mdblist_media(
        &self,
        api_key: String,
        media_type: &'static str,
        ids: Vec<String>,
    ) -> std::sync::mpsc::Receiver<Vec<Value>> {
        detached(async move {
            let Some(plan) = core_value(
                "mdblistMediaInfoBatchPlan",
                json!({"provider": "imdb", "mediaType": media_type, "ids": ids}),
            ) else {
                return Vec::new();
            };
            let Some(url) = plan.get("url").and_then(Value::as_str) else {
                return Vec::new();
            };
            let Ok(client) = Client::builder()
                .native_timeout(Duration::from_secs(15))
                .build()
            else {
                return Vec::new();
            };
            let response = client
                .post(format!("{url}?apikey={api_key}"))
                .header("Content-Type", "application/json")
                .body(plan.get("body").cloned().unwrap_or_default().to_string())
                .send()
                .await;
            let Ok(response) = response.and_then(|response| response.error_for_status()) else {
                return Vec::new();
            };
            response
                .json::<Value>()
                .await
                .ok()
                .and_then(|body| body.as_array().cloned())
                .unwrap_or_default()
        })
    }
}

pub(super) async fn tmdb_similar(
    client: &Client,
    item: &Value,
    api_key: &str,
    language: &str,
    endpoints: &[&str],
) -> Option<Vec<Value>> {
    if api_key.trim().is_empty() || endpoints.is_empty() {
        return None;
    }
    let content_type = item.get("type").and_then(Value::as_str).unwrap_or("movie");
    let request = |extra: Value| {
        let mut args = json!({
            "contentType": content_type,
            "contentId": item.get("id"),
            "language": language,
            "apiKey": api_key,
            "endpoints": endpoints,
        });
        if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
            args.extend(extra.clone());
        }
        args
    };
    let mut plan = core_value("tmdbDetailRequestPlan", request(json!({})))?;
    if plan.get("urls").is_none() {
        let find_url = plan.get("findUrl")?.as_str()?.to_owned();
        let find = fetch_json(client, &find_url).await.ok()?;
        plan = core_value(
            "tmdbDetailRequestUrlsFromFind",
            request(json!({"find": find})),
        )?;
    }
    for endpoint in endpoints {
        let Some(url) = plan
            .pointer(&format!("/urls/{endpoint}"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let Ok(response) = fetch_json(client, url).await else {
            continue;
        };
        let Some(results) = response
            .get("results")
            .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
        else {
            continue;
        };
        let metas = core_value(
            "tmdbBulkMetas",
            json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
        )
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
        if !metas.is_empty() {
            return Some(metas);
        }
    }
    None
}

pub(super) async fn tmdb_trailers(
    client: &Client,
    item: &Value,
    api_key: &str,
    language: &str,
) -> Option<Value> {
    let request = |extra: Value| {
        let mut args = json!({
            "contentType": item.get("type"),
            "contentId": item.get("id"),
            "language": language,
            "apiKey": api_key,
            "endpoints": ["videos"],
        });
        if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
            args.extend(extra.clone());
        }
        args
    };
    let mut plan = core_value("tmdbDetailRequestPlan", request(json!({})))?;
    if plan.get("urls").is_none() {
        let find_url = plan.get("findUrl")?.as_str()?.to_owned();
        let find = fetch_json(client, &find_url).await.ok()?;
        plan = core_value(
            "tmdbDetailRequestUrlsFromFind",
            request(json!({"find": find})),
        )?;
    }
    let url = plan.pointer("/urls/videos")?.as_str()?.to_owned();
    let videos = fetch_json(client, &url).await.ok()?;
    let results = videos
        .get("results")
        .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))?;
    core_value("tmdbBulkVideosToTrailers", results.clone())
}

pub(super) async fn youtube_request(payload: &Value) -> Result<Value, String> {
    let url = payload
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing trailer request URL".to_owned())?;
    let client = Client::builder()
        .native_timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?;
    let mut request = match payload.get("method").and_then(Value::as_str) {
        Some("POST") => client.post(url),
        _ => client.get(url),
    };
    for (name, value) in payload
        .get("headers")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if let Some(value) = value.as_str() {
            request = request.header(name.as_str(), value);
        }
    }
    if let Some(body) = payload.get("body").filter(|body| !body.is_null()) {
        request = request
            .header("Content-Type", "application/json")
            .body(body.to_string());
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok(json!({"statusCode": status, "body": body}))
}
