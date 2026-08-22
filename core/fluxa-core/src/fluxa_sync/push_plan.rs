use super::documents::hash_payload;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

fn slot(entity: &str, key: &str) -> String {
    format!("{entity}|{key}")
}

fn split_slot(value: &str) -> Option<(&str, &str)> {
    value.split_once('|')
}

fn change(entity: &str, key: &str, payload: Value, deleted: bool, expected: Option<i64>) -> Value {
    let mut fields = Map::new();
    fields.insert("entity_type".into(), json!(entity));
    fields.insert("key".into(), json!(key));
    fields.insert("payload".into(), payload);
    fields.insert("deleted".into(), json!(deleted));
    if let Some(revision) = expected {
        fields.insert("expected_revision".into(), json!(revision));
    }
    Value::Object(fields)
}

pub(crate) fn push_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let documents = args.get("documents")?.as_array()?;
    let known = args
        .get("known")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut changes: Vec<Value> = Vec::new();
    let mut present: BTreeSet<String> = BTreeSet::new();

    for document in documents {
        let (Some(entity), Some(key)) = (
            document.get("entity_type").and_then(Value::as_str),
            document.get("key").and_then(Value::as_str),
        ) else {
            continue;
        };
        let payload = document.get("payload").cloned().unwrap_or(Value::Null);
        let name = slot(entity, key);
        present.insert(name.clone());
        let previous = known.get(&name);
        let unchanged = previous
            .and_then(|entry| entry.get("hash"))
            .and_then(Value::as_str)
            == Some(hash_payload(&payload).as_str());
        if unchanged {
            continue;
        }
        let expected = previous
            .and_then(|entry| entry.get("revision"))
            .and_then(Value::as_i64);
        changes.push(change(entity, key, payload, false, expected));
    }

    for (name, entry) in &known {
        if present.contains(name) {
            continue;
        }
        let Some((entity, key)) = split_slot(name) else {
            continue;
        };
        let expected = entry.get("revision").and_then(Value::as_i64);
        changes.push(change(entity, key, Value::Null, true, expected));
    }

    serde_json::to_string(&json!({ "changes": changes })).ok()
}

pub(crate) fn apply_push_result_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut known = args
        .get("known")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let sent = args
        .get("changes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for entry in args
        .get("applied")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let (Some(entity), Some(key)) = (
            entry.get("entity_type").and_then(Value::as_str),
            entry.get("key").and_then(Value::as_str),
        ) else {
            continue;
        };
        let name = slot(entity, key);
        if entry.get("deleted").and_then(Value::as_bool) == Some(true) {
            known.remove(&name);
            continue;
        }
        let payload = sent
            .iter()
            .find(|change| {
                change.get("entity_type").and_then(Value::as_str) == Some(entity)
                    && change.get("key").and_then(Value::as_str) == Some(key)
            })
            .and_then(|change| change.get("payload"))
            .cloned()
            .unwrap_or(Value::Null);
        known.insert(
            name,
            json!({
                "revision": entry.get("revision").and_then(Value::as_i64).unwrap_or_default(),
                "hash": hash_payload(&payload),
            }),
        );
    }

    let stale = args
        .get("conflicts")
        .and_then(Value::as_array)
        .map(|conflicts| {
            conflicts
                .iter()
                .filter_map(|conflict| {
                    let entity = conflict.get("entity_type").and_then(Value::as_str)?;
                    let key = conflict.get("key").and_then(Value::as_str)?;
                    Some(json!({ "entity_type": entity, "key": key }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    serde_json::to_string(&json!({ "known": Value::Object(known), "stale": stale })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn changes_of(result: &str) -> Vec<Value> {
        let parsed: Value = serde_json::from_str(result).expect("valid json");
        parsed
            .get("changes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }

    #[test]
    fn a_document_that_disappeared_locally_is_pushed_as_a_deletion() {
        let result = push_plan_json(
            &json!({
                "documents": [],
                "known": { "collections|abc": { "revision": 7, "hash": "deadbeef" } },
            })
            .to_string(),
        )
        .expect("plan");

        let changes = changes_of(&result);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].get("deleted"), Some(&json!(true)));
        assert_eq!(changes[0].get("key"), Some(&json!("abc")));
        assert_eq!(changes[0].get("expected_revision"), Some(&json!(7)));
    }

    #[test]
    fn an_unchanged_payload_is_not_pushed_again() {
        let payload = json!({ "title": "Movies" });
        let documents = json!([{ "entity_type": "collections", "key": "abc", "payload": payload }]);
        let known = json!({
            "collections|abc": { "revision": 3, "hash": hash_payload(&payload) },
        });

        let result =
            push_plan_json(&json!({ "documents": documents, "known": known }).to_string())
                .expect("plan");

        assert!(changes_of(&result).is_empty());
    }
}
