use super::*;

impl EffectExecutor {
    pub(super) async fn fetch_plugin_manifest(&self, payload: &Value) -> Result<Value, String> {
        let manifest_url = payload
            .get("manifestUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| "plugin repository is missing manifestUrl".to_owned())?;
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let (status, body) = fetch_text(&client, manifest_url).await?;
        let body = body
            .filter(|_| (200..300).contains(&status))
            .ok_or_else(|| format!("plugin repository returned HTTP {status}"))?;
        let manifest: Value = serde_json::from_str(&body)
            .map_err(|_| "plugin manifest is not valid JSON".to_owned())?;
        if !manifest.get("scrapers").is_some_and(Value::is_array) {
            return Err("plugin manifest has no scrapers".to_owned());
        }
        Ok(json!({"manifestUrl": manifest_url, "manifest": manifest}))
    }

    pub async fn nuvio_plugin_urls(&self) -> Result<Option<Vec<String>>, String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(None);
        };
        let scope = self.nuvio_profile_scope(&session, "plugins").await;
        let rows = session
            .rest(&format!(
                "plugins?select=*&profile_id=eq.{scope}&order=sort_order"
            ))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("plugin list request failed: {error}"))?
            .json::<Value>()
            .await
            .map_err(|error| format!("plugin list response invalid: {error}"))?;
        Ok(Some(
            rows.as_array()
                .into_iter()
                .flatten()
                .filter(|row| row.get("enabled").and_then(Value::as_bool) != Some(false))
                .filter_map(|row| row.get("url").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect(),
        ))
    }

    pub async fn push_nuvio_plugins(
        &self,
        repositories: Vec<(String, String)>,
    ) -> Result<(), String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(());
        };
        let scope = self.nuvio_profile_scope(&session, "plugins").await;
        let plugins = repositories
            .into_iter()
            .enumerate()
            .map(|(index, (url, name))| {
                json!({"url": url, "name": name, "enabled": true, "sort_order": index})
            })
            .collect::<Vec<_>>();
        let response = session
            .rpc("sync_push_plugins")
            .json(&json!({"p_profile_id": scope, "p_plugins": plugins}))
            .send()
            .await
            .map_err(|error| format!("plugin push failed: {error}"))?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(format!("Nuvio rejected the plugin list ({status}): {body}"))
    }
}
