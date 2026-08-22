use super::helpers::{meta_text, year_regex};
use serde_json::Value;

pub(crate) fn meta_year(meta: &Value) -> Option<String> {
    ["released", "releaseInfo"].into_iter().find_map(|key| {
        year_regex()
            .find(meta_text(meta, key))
            .map(|matched| matched.as_str().to_string())
    })
}

pub(crate) fn rating_value(value: &Value) -> Option<f32> {
    let text = match value {
        Value::String(text) => text.trim().to_string(),
        Value::Number(number) => number.to_string(),
        _ => value.to_string().trim_matches('"').trim().to_string(),
    };
    if text.is_empty() {
        return None;
    }
    if let Some((score, scale)) = text.split_once('/') {
        let score = score.trim().parse::<f32>().ok()?;
        let scale = scale.trim().parse::<f32>().ok()?;
        if scale == 0.0 {
            None
        } else {
            Some((score / scale) * 10.0)
        }
    } else if let Some(percent) = text.strip_suffix('%') {
        percent.trim().parse::<f32>().ok().map(|value| value / 10.0)
    } else {
        text.parse::<f32>().ok()
    }
}

pub(crate) fn meta_rating(meta: &Value) -> Option<f32> {
    meta.get("imdbRating")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<f32>().ok())
        .or_else(|| {
            meta.get("ratings")
                .and_then(Value::as_array)
                .and_then(|ratings| {
                    ratings
                        .iter()
                        .find_map(|rating| rating.get("value").and_then(rating_value))
                })
        })
}

pub(crate) fn matches_discover_year(meta: &Value, year: Option<&str>) -> bool {
    let Some(expected) = year.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };
    meta_year(meta).as_deref() == Some(expected)
}

pub(crate) fn matches_discover_rating(meta: &Value, minimum_rating: Option<f32>) -> bool {
    let Some(minimum) = minimum_rating else {
        return true;
    };
    meta_rating(meta).is_some_and(|candidate| candidate >= minimum)
}

pub(crate) fn matches_discover_region(meta: &Value, region: Option<&str>) -> bool {
    let Some(expected) = region
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
    else {
        return true;
    };
    let language = meta_text(meta, "originalLanguage").to_lowercase();
    if language.is_empty() {
        return false;
    }
    match expected.as_str() {
        "us" | "usa" | "en" => language == "en",
        "jp" | "ja" | "japan" => language == "ja",
        "kr" | "ko" | "korea" => language == "ko",
        _ => language == expected,
    }
}

pub(crate) fn filter_discover_results_json(
    items_json: &str,
    year: Option<&str>,
    rating: Option<f32>,
    region: Option<&str>,
) -> Option<String> {
    let items = serde_json::from_str::<Vec<Value>>(items_json).ok()?;
    let filtered = items
        .into_iter()
        .filter(|item| matches_discover_year(item, year))
        .filter(|item| matches_discover_rating(item, rating))
        .filter(|item| matches_discover_region(item, region))
        .collect::<Vec<_>>();
    serde_json::to_string(&filtered).ok()
}
