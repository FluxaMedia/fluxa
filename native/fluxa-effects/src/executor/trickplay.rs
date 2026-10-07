use super::*;

impl EffectExecutor {
    pub fn resolve_trickplay_source(
        &self,
        payload: Value,
    ) -> std::sync::mpsc::Receiver<Option<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let executor = self.clone();
        let task = async move {
            let _ = sender.send(executor.trickplay_source(&payload).await);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }

    async fn trickplay_source(&self, payload: &Value) -> Option<Value> {
        let text = |value: &Value, key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        let content_type = text(payload, "contentType");
        let request_ids =
            streams::stream_request_ids(&content_type, &text(payload, "videoId"), None).ok()?;
        let source = payload.get("source")?;
        let id = text(source, "id");
        let (addons, scrapers) = match text(source, "kind").as_str() {
            "addon" => {
                let all = self.account_addons().await.ok()?;
                let chosen = all
                    .as_array()?
                    .iter()
                    .filter(|addon| {
                        text(addon, "transportUrl") == id
                            || text(&addon["manifest"], "transportUrl") == id
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                (json!(chosen), json!([]))
            }
            "plugin" => (json!([]), json!([id])),
            _ => return None,
        };
        let streams = self
            .fetch_streams(&json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": request_ids,
                "addons": addons,
                "onlyScrapers": scrapers,
            }))
            .await
            .ok()?;
        core_value(
            "trickplayPickSource",
            json!({"streams": streams, "pattern": text(payload, "regex")}),
        )
        .filter(|picked| !picked.is_null())
    }
}
