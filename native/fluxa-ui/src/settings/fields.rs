use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets, Justify};



pub fn setup_accepts(field_key: &str, pref: &str) -> bool {
    let lower = pref.to_lowercase();
    if ["key", "token", "password", "secret", "epoch"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return false;
    }
    match field_key {
        "metadatakitSetup" => pref.starts_with("metadatakit") || pref.starts_with("catalog"),
        _ => {
            (pref.starts_with("stream") && !pref.starts_with("streamProxy"))
                || pref.starts_with("debrid")
        }
    }
}

pub fn setup_export(field_key: &str, values: &serde_json::Value) -> String {
    let picked: serde_json::Map<String, serde_json::Value> = values
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(pref, value)| {
            setup_accepts(field_key, pref) && !value.is_array() && !value.is_object()
        })
        .map(|(pref, value)| (pref.clone(), value.clone()))
        .collect();
    serde_json::Value::Object(picked).to_string()
}





