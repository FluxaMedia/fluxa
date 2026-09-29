use serde_json::Value;

pub(crate) fn annotate_catalog_items_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let items_json = request.get("itemsJson")?.as_str()?;
    let transport_url = request.get("transportUrl").cloned().unwrap_or(Value::Null);
    let catalog_type = request.get("catalogType").cloned().unwrap_or(Value::Null);
    let mut items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    for item in &mut items {
        if let Some(fields) = item.as_object_mut() {
            fields.insert("sourceAddonTransportUrl".to_owned(), transport_url.clone());
            fields.insert("sourceAddonCatalogType".to_owned(), catalog_type.clone());
        }
    }
    serde_json::to_string(&items).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn catalog_source_context_is_attached_by_core_without_dropping_meta_fields() {
        let result: Value = serde_json::from_str(
            &annotate_catalog_items_json(
                &json!({
                    "itemsJson": json!([{"id":"tt1", "name":"Movie"}]).to_string(),
                    "transportUrl": "https://addon.example/manifest.json",
                    "catalogType": "movie"
                })
                .to_string(),
            )
            .expect("annotated catalog items"),
        )
        .expect("valid JSON");
        assert_eq!(result[0]["id"], "tt1");
        assert_eq!(
            result[0]["sourceAddonTransportUrl"],
            "https://addon.example/manifest.json"
        );
        assert_eq!(result[0]["sourceAddonCatalogType"], "movie");
    }
}
