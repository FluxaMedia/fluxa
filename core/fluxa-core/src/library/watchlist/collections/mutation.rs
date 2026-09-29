use serde_json::{Value, json};

pub(crate) fn collection_mutation_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let mut collections = request.get("collections")?.as_array()?.clone();
    let command = request.get("command")?.as_object()?;
    let command_type = command.get("type")?.as_str()?;

    match command_type {
        "create" | "upsert" => {
            let collection = command.get("collection")?.clone();
            let id = collection.get("id")?.as_str()?.to_string();
            let title = collection.get("title")?.as_str()?.trim();
            if id.is_empty() || title.is_empty() {
                return None;
            }

            if let Some(object) = collection.as_object() {
                let mut normalized = object.clone();
                normalized.insert("title".into(), Value::String(title.to_string()));
                let collection = Value::Object(normalized);
                if let Some(index) = collections
                    .iter()
                    .position(|entry| entry.get("id").and_then(Value::as_str) == Some(&id))
                {
                    collections[index] = collection;
                } else {
                    collections.push(collection);
                }
            }
        }
        "rename" => {
            let id = command.get("id")?.as_str()?;
            let title = command.get("title")?.as_str()?.trim();
            if id.is_empty() || title.is_empty() {
                return None;
            }
            if let Some(collection) = collections
                .iter_mut()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(id))
            {
                collection["title"] = Value::String(title.to_string());
            }
        }
        "delete" => {
            let id = command.get("id")?.as_str()?;
            collections.retain(|entry| entry.get("id").and_then(Value::as_str) != Some(id));
        }
        "replace" => {
            collections = command.get("collections")?.as_array()?.clone();
        }
        "saveFolder" => {
            let collection_id = command.get("collectionId")?.as_str()?;
            let folder = command.get("folder")?.clone();
            let folder_id = folder.get("id")?.as_str()?.to_string();
            let title = folder.get("title")?.as_str()?.trim().to_string();
            if collection_id.is_empty() || folder_id.is_empty() || title.is_empty() {
                return None;
            }
            let Some(collection) = collections
                .iter_mut()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(collection_id))
            else {
                return None;
            };
            let folders = collection
                .get("folders")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut next_folder = folder;
            next_folder["title"] = Value::String(title);
            let mut next_folders = folders;
            if let Some(index) = next_folders.iter().position(|entry| {
                entry.get("id").and_then(Value::as_str) == Some(folder_id.as_str())
            }) {
                next_folders[index] = next_folder;
            } else {
                next_folders.push(next_folder);
            }
            collection["folders"] = Value::Array(next_folders);
        }
        "deleteFolder" => {
            let collection_id = command.get("collectionId")?.as_str()?;
            let folder_id = command.get("folderId")?.as_str()?;
            if collection_id.is_empty() || folder_id.is_empty() {
                return None;
            }
            let Some(collection) = collections
                .iter_mut()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(collection_id))
            else {
                return None;
            };
            let next_folders = collection
                .get("folders")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(folder_id))
                .collect();
            collection["folders"] = Value::Array(next_folders);
        }
        _ => return None,
    }

    Some(json!({"collections": collections}).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn collections(request: &str) -> Value {
        serde_json::from_str::<Value>(&collection_mutation_plan_json(request).unwrap()).unwrap()
            ["collections"]
            .clone()
    }

    #[test]
    fn upsert_replaces_matching_collection_without_reordering() {
        let result = collections(
            r#"{"collections":[{"id":"a","title":"Old"},{"id":"b","title":"B"}],"command":{"type":"upsert","collection":{"id":"a","title":"New"}}}"#,
        );
        assert_eq!(result[0]["title"], "New");
        assert_eq!(result[1]["id"], "b");
    }

    #[test]
    fn folder_mutations_are_applied_in_core() {
        let result = collections(
            r#"{"collections":[{"id":"a","title":"A","folders":[{"id":"f","title":"Old"}]}],"command":{"type":"saveFolder","collectionId":"a","folder":{"id":"f","title":"New"}}}"#,
        );
        assert_eq!(result[0]["folders"][0]["title"], "New");

        let result = collections(
            r#"{"collections":[{"id":"a","title":"A","folders":[{"id":"f","title":"Old"}]}],"command":{"type":"deleteFolder","collectionId":"a","folderId":"f"}}"#,
        );
        assert!(result[0]["folders"].as_array().unwrap().is_empty());
    }
}
