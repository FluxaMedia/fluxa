use super::*;

impl EffectExecutor {
    async fn direct_streams_request(&self, meta: &Value) -> Result<Value, String> {
        let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("movie");
        let id = meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .or_else(|| meta.get("id").and_then(Value::as_str))
            .or_else(|| meta.get("imdb_id").and_then(Value::as_str))
            .ok_or_else(|| "direct playback meta is missing id".to_owned())?;
        let request_ids = stream_request_ids(content_type, id, Some(meta))?;
        let mut request = json!({
            "kind": "streams",
            "resource": "stream",
            "contentType": content_type,
            "requestIds": request_ids,
            "addons": self.account_addons().await?,
        });
        request["detail"] = meta.clone();
        Ok(request)
    }

    pub fn stream_feed_since(&self, opened_ms: u128, version: u64) -> Option<FeedSnapshot> {
        let feed = self.feed.lock().ok()?;
        (feed.started_ms >= opened_ms && feed.version != version).then(|| FeedSnapshot {
            version: feed.version,
            parts: feed.parts.clone(),
            done: feed.done,
        })
    }

    fn begin_stream_feed(&self) -> u64 {
        let mut feed = self.feed.lock().unwrap_or_else(|error| error.into_inner());
        feed.generation += 1;
        feed.version += 1;
        feed.parts.clear();
        feed.done = false;
        feed.started_ms = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        feed.generation
    }

    fn feed_progress(&self, generation: u64) -> Progress {
        let feed = self.feed.clone();
        std::sync::Arc::new(move |index, streams| {
            let mut feed = feed.lock().unwrap_or_else(|error| error.into_inner());
            if feed.generation == generation {
                feed.parts.insert(index, streams);
                feed.version += 1;
            }
        })
    }

    fn finish_stream_feed(&self, generation: u64) {
        let mut feed = self.feed.lock().unwrap_or_else(|error| error.into_inner());
        if feed.generation == generation {
            feed.done = true;
            feed.version += 1;
        }
    }

    pub(super) async fn prepare_direct_playback(&self, payload: &Value) -> Result<Value, String> {
        let meta = payload
            .get("meta")
            .filter(|value| value.is_object())
            .ok_or_else(|| "direct playback is missing meta".to_owned())?;
        let request = self.direct_streams_request(meta).await?;
        let id = meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .or_else(|| meta.get("id").and_then(Value::as_str))
            .or_else(|| meta.get("imdb_id").and_then(Value::as_str))
            .unwrap_or_default();
        let manual = meta.get("pickSource").is_some()
            || payload
                .pointer("/profile/streamSourceSelectionMode")
                .and_then(Value::as_str)
                .unwrap_or("manual")
                == "manual";
        let streams = if manual {
            let generation = self.begin_stream_feed();
            let result = self
                .fetch_streams_into(&request, Some(self.feed_progress(generation)))
                .await;
            self.finish_stream_feed(generation);
            result?
        } else {
            self.fetch_streams(&request).await?
        };
        let has_stream_providers = !streams.is_empty();
        let available_addons = core_value(
            "detailStreamResultPlan",
            json!({
                "attempts": [{"requestId": id, "streams": streams.clone()}],
                "hasStreamProviders": has_stream_providers
            }),
        )
        .and_then(|plan| plan.get("availableAddons").cloned())
        .unwrap_or_else(|| json!([]));
        Ok(json!({
            "streams": streams,
            "availableAddons": available_addons,
            "failedAddons": [],
            "hasStreamProviders": has_stream_providers,
        }))
    }

    pub(super) async fn load_streams(&self, payload: &Value) -> Result<Value, String> {
        if payload.get("useInitialStreams").and_then(Value::as_bool) == Some(true) {
            let streams = payload
                .get("initialStreams")
                .and_then(Value::as_array)
                .filter(|streams| !streams.is_empty())
                .ok_or_else(|| "Core requested initial streams but supplied none".to_owned())?;
            return Ok(Value::Array(streams.clone()));
        }

        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "loadStreams is missing id".to_owned())?;
        let request_ids = stream_request_ids(content_type, id, None)?;
        let streams = self
            .fetch_streams(&json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": request_ids,
                "addons": self.account_addons().await?,
            }))
            .await?;
        Ok(Value::Array(streams))
    }

    pub(super) async fn prefetch_next_episode_streams(
        &self,
        payload: &Value,
    ) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("series");
        let next_video_id = payload
            .get("nextVideoId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "prefetchNextEpisodeStreams is missing nextVideoId".to_owned())?;
        let request_ids = stream_request_ids(content_type, next_video_id, None)?;
        let streams = self
            .fetch_streams(&json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": request_ids,
                "addons": self.account_addons().await?,
            }))
            .await?;
        Ok(json!({ "streams": streams }))
    }

    pub(super) async fn fetch_streams(&self, request: &Value) -> Result<Vec<Value>, String> {
        self.fetch_streams_into(request, None).await
    }

    async fn fetch_streams_into(
        &self,
        request: &Value,
        progress: Option<Progress>,
    ) -> Result<Vec<Value>, String> {
        let policy = core_value("resourceFetchExecutionPolicy", request.clone())
            .ok_or_else(|| "Fluxa Core could not create a stream fetch plan".to_owned())?;
        let requests = policy
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        #[cfg(not(target_arch = "wasm32"))]
        let scrapers = self.enabled_scrapers(request).await;
        #[cfg(target_arch = "wasm32")]
        let scrapers: Vec<Value> = Vec::new();
        if requests.is_empty() && scrapers.is_empty() {
            return Ok(Vec::new());
        }
        let request_count = requests.len();
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(25))
            .build()
            .map_err(|error| error.to_string())?;
        let mut jobs: Vec<JobFuture> = Vec::new();
        for (index, item) in requests.into_iter().enumerate() {
            let client = client.clone();
            let request_resource = request.get("resource").cloned();
            let progress = progress.clone();
            jobs.push(Box::pin(async move {
                let Some(url) = item.get("url").and_then(Value::as_str) else {
                    return (index, Vec::new());
                };
                let kind = item
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("streams");
                let resource = core_value(
                    "resourceKindToResource",
                    json!({
                        "kind": kind,
                        "requestResource": request_resource,
                        "itemResource": item.get("resource"),
                    }),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "stream".to_owned());
                let (status_code, body) = match fetch_text(&client, url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] stream request failed for {}: {error}",
                            url_without_query(url)
                        );
                        (0, None)
                    }
                };
                let streams = core_value(
                    "parseAndPlanAddonResource",
                    json!({
                        "resource": resource,
                        "url": url,
                        "statusCode": status_code,
                        "body": body,
                        "kind": kind,
                        "addonName": item.get("addonName"),
                        "season": Value::Null,
                    }),
                )
                .and_then(|parsed| {
                    (parsed.get("kind").and_then(Value::as_str) == Some("success")).then(|| {
                        parsed
                            .get("value")
                            .and_then(|value| value.get("streams"))
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default();
                if let Some(progress) = &progress {
                    progress(index, streams.clone());
                }
                (index, streams)
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        if !scrapers.is_empty() {
            use futures::FutureExt;
            let resolver = self.clone();
            let lookup = request.clone();
            let target = async move {
                let target = resolver.plugin_target(&lookup).await;
                if target.is_none() {
                    crate::log!(
                        "[fluxa-native] plugin target unresolved: no TMDB id for this title"
                    );
                }
                target.map(std::sync::Arc::new)
            }
            .boxed()
            .shared();
            for (offset, scraper) in scrapers.into_iter().enumerate() {
                let executor = self.clone();
                let target = target.clone();
                let progress = progress.clone();
                let index = request_count + offset;
                jobs.push(Box::pin(async move {
                    let streams = match target.await {
                        Some(target) => executor.run_plugin_scraper(scraper, target).await,
                        None => Vec::new(),
                    };
                    if let Some(progress) = &progress {
                        progress(index, streams.clone());
                    }
                    (index, streams)
                }));
            }
        }

        let mut results = if progress.is_some() {
            futures::future::join_all(jobs).await
        } else {
            collect_with_grace(jobs).await
        };
        results.sort_by_key(|(index, _)| *index);
        Ok(results
            .into_iter()
            .flat_map(|(_, streams)| streams)
            .collect())
    }

    pub(super) async fn fetch_detail_streams(&self, payload: &Value) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let request_ids = payload
            .get("requestIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .collect::<Vec<_>>();
        let addons = self.account_addons().await?;
        let provider_plan = core_value(
            "providerAvailabilityPlan",
            json!({"addons": addons.clone(), "pluginNames": []}),
        )
        .ok_or_else(|| "Fluxa Core could not resolve stream provider availability".to_owned())?;
        let mut attempts = Vec::new();
        for request_id in request_ids {
            let mut request = json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": [request_id],
                "addons": addons.clone()
            });
            if let Some(detail) = payload.get("detail") {
                request["detail"] = detail.clone();
            }
            let streams = self.fetch_streams(&request).await?;
            let found_streams = !streams.is_empty();
            attempts.push(json!({"requestId": request_id, "streams": streams}));
            if found_streams {
                break;
            }
        }
        core_value(
            "detailStreamResultPlan",
            json!({
                "attempts": attempts,
                "hasStreamProviders": provider_plan
                    .get("hasStreamProviders")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            }),
        )
        .ok_or_else(|| "Fluxa Core could not build the detail stream result".to_owned())
    }
}

pub(super) fn stream_request_ids(
    content_type: &str,
    id: &str,
    detail: Option<&Value>,
) -> Result<Vec<String>, String> {
    let detail_id = detail
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str);
    let current_series_lookup_id = detail_id.and_then(|detail_id| {
        (content_type == "series" || content_type == "tv" || content_type == "show")
            .then(|| detail_id.to_owned())
    });
    core_value(
        "streamRequestIds",
        json!({
            "contentType": content_type,
            "id": id,
            "detailId": detail_id,
            "currentSeriesLookupId": current_series_lookup_id,
        }),
    )
    .and_then(|value| value.as_array().cloned())
    .map(|values| {
        values
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
    })
    .filter(|values| !values.is_empty())
    .ok_or_else(|| "Fluxa Core could not resolve stream request IDs".to_owned())
}

#[cfg(target_arch = "wasm32")]
async fn collect_with_grace<F>(jobs: Vec<F>) -> Vec<(usize, Vec<Value>)>
where
    F: std::future::Future<Output = (usize, Vec<Value>)>,
{
    futures::future::join_all(jobs).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn collect_with_grace<F>(jobs: Vec<F>) -> Vec<(usize, Vec<Value>)>
where
    F: std::future::Future<Output = (usize, Vec<Value>)>,
{
    use futures::StreamExt;
    const GRACE: Duration = Duration::from_millis(2500);
    let mut pending = jobs
        .into_iter()
        .collect::<futures::stream::FuturesUnordered<_>>();
    let mut results = Vec::new();
    let mut deadline = None;
    loop {
        let next = match deadline {
            Some(deadline) => match tokio::time::timeout_at(deadline, pending.next()).await {
                Ok(next) => next,
                Err(_) => break,
            },
            None => pending.next().await,
        };
        let Some((index, streams)) = next else {
            break;
        };
        if deadline.is_none() && !streams.is_empty() {
            deadline = Some(tokio::time::Instant::now() + GRACE);
        }
        results.push((index, streams));
    }
    results
}

#[cfg(not(target_arch = "wasm32"))]
type JobFuture = futures::future::BoxFuture<'static, (usize, Vec<Value>)>;
#[cfg(target_arch = "wasm32")]
type JobFuture = futures::future::LocalBoxFuture<'static, (usize, Vec<Value>)>;

type Progress = std::sync::Arc<dyn Fn(usize, Vec<Value>) + Send + Sync>;

#[derive(Default)]
pub struct StreamFeed {
    generation: u64,
    version: u64,
    started_ms: u128,
    parts: std::collections::BTreeMap<usize, Vec<Value>>,
    done: bool,
}

pub struct FeedSnapshot {
    pub version: u64,
    pub parts: std::collections::BTreeMap<usize, Vec<Value>>,
    pub done: bool,
}
