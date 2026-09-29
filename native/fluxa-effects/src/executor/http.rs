use super::*;

impl EffectExecutor {
    pub fn request_json(&self, plan: Value) -> std::sync::mpsc::Receiver<Option<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let _ = sender.send(send_plan(&plan).await);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        super::runtime().spawn(task);
        receiver
    }
}

async fn send_plan(plan: &Value) -> Option<Value> {
    let url = reqwest::Url::parse(plan.get("url")?.as_str()?).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let method = plan.get("method").and_then(Value::as_str).unwrap_or("GET");
    let client = Client::builder()
        .native_timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let mut request = client
        .request(reqwest::Method::from_bytes(method.as_bytes()).ok()?, url)
        .header("User-Agent", "Fluxa");
    for (name, value) in plan.get("headers").and_then(Value::as_object)? {
        request = request.header(name.as_str(), value.as_str()?);
    }
    if let Some(body) = plan.get("body").and_then(Value::as_str) {
        request = request.body(body.to_owned());
    }
    request
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Value>()
        .await
        .ok()
}
