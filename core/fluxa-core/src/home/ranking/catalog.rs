use serde_json::Value;

pub fn annotate_catalog_items(
    mut items: Vec<Value>,
    transport_url: &str,
    catalog_type: &str,
) -> Vec<Value> {
    for item in &mut items {
        if let Some(fields) = item.as_object_mut() {
            fields.insert(
                "sourceAddonTransportUrl".to_owned(),
                Value::String(transport_url.to_owned()),
            );
            fields.insert(
                "sourceAddonCatalogType".to_owned(),
                Value::String(catalog_type.to_owned()),
            );
        }
    }
    items
}

pub(crate) fn annotate_catalog_items_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let items: Vec<Value> = serde_json::from_str(request.get("itemsJson")?.as_str()?).ok()?;
    let text = |key| request.get(key).and_then(Value::as_str).unwrap_or_default();
    serde_json::to_string(&annotate_catalog_items(
        items,
        text("transportUrl"),
        text("catalogType"),
    ))
    .ok()
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
