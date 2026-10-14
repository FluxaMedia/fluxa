use super::{EffectExecutor, client_id, http_client, str_field};
use crate::executor::{core_value, tmdb};
use reqwest::Client;
use serde_json::{Value, json};

const TRACKERS: [&str; 2] = ["trakt", "simkl"];

impl EffectExecutor {
    pub(in crate::executor) async fn fetch_detail_secondary(
        &self,
        payload: &Value,
    ) -> Result<Value, String> {
        let profile = payload.get("profile").cloned().unwrap_or(Value::Null);
        let content_type = str_field(payload, "contentType");
        let id = str_field(payload, "id");
        let language = str_field(payload, "language");
        let chosen = Some(str_field(payload, "similarTitlesSource"))
            .filter(|source| !source.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                self.profile_prefs(str_field(&profile, "id"))
                    .ok()
                    .flatten()?
                    .get("similarTitlesSource")?
                    .as_str()
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let chosen = chosen.as_str();
        let library = str_field(&profile, "integrationLibrarySource");
        let order: Vec<&str> = match chosen {
            "tmdb" => vec!["tmdb"],
            "trakt" | "simkl" => vec![chosen, "tmdb"],
            _ => {
                let mut connected: Vec<&str> = TRACKERS
                    .into_iter()
                    .filter(|tracker| self.provider_credentials(tracker, &profile).is_some())
                    .collect();
                connected.sort_by_key(|tracker| *tracker != library);
                connected.extend(["tmdb", "trakt"]);
                let mut seen = Vec::new();
                connected.retain(|source| {
                    let first = !seen.contains(source);
                    seen.push(*source);
                    first
                });
                connected
            }
        };
        let client = http_client()?;
        for source in order {
            let items = if source == "tmdb" {
                self.tmdb_similar_items(&profile, content_type, id, language)
                    .await
            } else {
                self.tracker_similar_items(&client, source, &profile, content_type, id)
                    .await
            };
            if !items.is_empty() {
                return Ok(json!({"similarItems": items}));
            }
        }
        Ok(json!({"similarItems": []}))
    }

    async fn tmdb_similar_items(
        &self,
        profile: &Value,
        content_type: &str,
        id: &str,
        language: &str,
    ) -> Vec<Value> {
        let api_key = self.provider_pref(profile, "tmdbApiKey");
        let prefs = self
            .profile_prefs(str_field(profile, "id"))
            .ok()
            .flatten()
            .unwrap_or(Value::Null);
        let on = |key: &str| prefs.get(key).and_then(Value::as_bool).unwrap_or(true);
        let endpoints: Vec<&'static str> = [
            ("recommendations", on("tmdbRecommendationsEnabled")),
            ("similar", on("tmdbSimilarResultsEnabled")),
        ]
        .into_iter()
        .filter_map(|(endpoint, enabled)| enabled.then_some(endpoint))
        .collect();
        let Ok(client) = http_client() else {
            return Vec::new();
        };
        let language = if language.is_empty() { "en" } else { language };
        let item = json!({"type": content_type, "id": id});
        tmdb::tmdb_similar(&client, &item, &api_key, language, &endpoints)
            .await
            .unwrap_or_default()
    }

    async fn tracker_similar_items(
        &self,
        client: &Client,
        provider: &str,
        profile: &Value,
        content_type: &str,
        id: &str,
    ) -> Vec<Value> {
        let credentials = self.provider_credentials(provider, profile).or_else(|| {
            (provider == "trakt" && !client_id(provider).is_empty()).then(
                || json!({"provider": provider, "token": "", "clientId": client_id(provider)}),
            )
        });
        let Some(mut credentials) = credentials else {
            return Vec::new();
        };
        credentials["contentType"] = json!(content_type);
        credentials["id"] = json!(id);
        let step = |extra: Value, credentials: &Value| {
            let mut args = credentials.clone();
            args["step"] = extra["step"].clone();
            if let Some(simkl) = extra.get("simklId") {
                args["simklId"] = simkl.clone();
            }
            args
        };
        let fetch = |args: Value| async move {
            let plan = core_value("detailSimilarPlan", args.clone())?;
            let response = Self::provider_get(client, &plan, provider).await.ok()?;
            let mut args = args;
            args["response"] = response;
            core_value("detailSimilarItems", args)
        };
        if provider == "trakt" {
            return fetch(step(json!({"step": "related"}), &credentials))
                .await
                .and_then(|items| items.as_array().cloned())
                .unwrap_or_default();
        }
        let Some(lookup) = fetch(step(json!({"step": "lookup"}), &credentials)).await else {
            return Vec::new();
        };
        let detail = step(
            json!({"step": "detail", "simklId": lookup["simklId"]}),
            &credentials,
        );
        let recommendations = fetch(detail)
            .await
            .and_then(|items| items.as_array().cloned())
            .unwrap_or_default();
        let resolved = futures::future::join_all(recommendations.iter().take(10).map(|card| {
            let args = step(
                json!({"step": "resolve", "simklId": card["simklId"]}),
                &credentials,
            );
            fetch(args)
        }))
        .await;
        let metas = futures::future::join_all(recommendations.iter().take(10).map(|card| {
            let args = step(
                json!({"step": "meta", "simklId": card["simklId"]}),
                &credentials,
            );
            fetch(args)
        }))
        .await;
        recommendations
            .into_iter()
            .zip(resolved)
            .zip(metas)
            .filter_map(|((mut card, resolved), meta)| {
                card["id"] = resolved?.get("id")?.clone();
                card["type"] = json!(content_type);
                if let (Some(card), Some(Value::Object(meta))) = (card.as_object_mut(), meta) {
                    card.extend(meta.into_iter().filter(|(_, value)| !value.is_null()));
                }
                Some(card)
            })
            .collect()
    }
}
