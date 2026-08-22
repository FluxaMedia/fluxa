use super::helpers::{meta_i64, meta_text};
use crate::content_identity::{imdb_id, normalized_billboard_title};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

const RANKED_CATALOG_IDS: &[&str] = &["trending", "popular", "top", "now_playing"];

fn has_backdrop_candidate(meta: &Value) -> bool {
    let background = meta_text(meta, "background");
    !background.is_empty() && !background.eq_ignore_ascii_case(meta_text(meta, "poster"))
}

fn score_candidate(meta: &Value, days_since_release: Option<i64>) -> i32 {
    let release_boost = match days_since_release {
        None => 0,
        Some(days) if days < 0 => 40,
        Some(days) if days <= 14 => 440,
        Some(days) if days <= 45 => 280,
        Some(days) if days <= 120 => 120,
        Some(_) => 0,
    };
    let type_boost = if meta_text(meta, "type") == "series" {
        320
    } else {
        140
    };
    let rank_boost = meta_i64(meta, "rank")
        .map(|rank| (220 - ((rank as i32 - 1) * 18)).max(0))
        .unwrap_or(0);
    let rating_boost = (meta_text(meta, "imdbRating").parse::<f32>().unwrap_or(0.0) * 22.0) as i32;
    let recommendation_boost = if meta_text(meta, "reason").is_empty() {
        0
    } else {
        180
    };
    let editorial_boost = if meta_text(meta, "reason") == "EDITORIAL_SPOTLIGHT" {
        520
    } else {
        0
    };
    let backdrop_boost = if has_backdrop_candidate(meta) {
        260
    } else if !meta_text(meta, "poster").is_empty() {
        40
    } else {
        -240
    };
    type_boost
        + release_boost
        + rank_boost
        + rating_boost
        + recommendation_boost
        + editorial_boost
        + backdrop_boost
}

fn billboard_key_value(meta: &Value) -> String {
    let id = meta_text(meta, "id");
    if let Some(iid) = imdb_id(id) {
        return format!("{}:{iid}", meta_text(meta, "type"));
    }
    let name = meta
        .get("originalName")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| meta_text(meta, "name"));
    let year = meta_text(meta, "releaseInfo")
        .get(0..4)
        .or_else(|| meta_text(meta, "released").get(0..4))
        .unwrap_or("");
    format!(
        "{}:{}:{year}",
        meta_text(meta, "type"),
        normalized_billboard_title(name)
    )
}

fn title_key_value(meta: &Value) -> String {
    let name = meta
        .get("originalName")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| meta_text(meta, "name"));
    normalized_billboard_title(name)
}

fn distinct_by_billboard_key(items: Vec<Value>) -> Vec<Value> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|m| seen.insert(billboard_key_value(m)))
        .collect()
}

fn distinct_by_title_key(items: Vec<Value>) -> Vec<Value> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|m| seen.insert(title_key_value(m)))
        .collect()
}

fn billboard_visual_score(meta: &Value) -> i32 {
    let mut score = 0i32;
    if has_backdrop_candidate(meta) {
        score += 320;
    } else {
        score -= 160;
    }
    if !meta_text(meta, "logo").is_empty() {
        score += 120;
    }
    if !meta_text(meta, "description").is_empty() {
        score += 30;
    }
    score
}

pub(crate) fn billboard_candidate_score_json(args_json: &str) -> Option<i32> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let days_since_release = args.get("daysSinceRelease").and_then(Value::as_i64);
    Some(score_candidate(meta, days_since_release))
}

pub(crate) fn billboard_visual_score_json(args_json: &str) -> Option<i32> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    Some(billboard_visual_score(args.get("meta")?))
}

pub(crate) fn billboard_has_backdrop_json(args_json: &str) -> Option<bool> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    Some(has_backdrop_candidate(args.get("meta")?))
}

pub(crate) fn billboard_editorial_match_score_json(args_json: &str) -> Option<i32> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let min_year = args.get("minYear")?.as_i64()? as i32;
    let release_year = meta_text(meta, "releaseInfo").parse::<i32>().unwrap_or(0);
    let year_boost = if release_year >= min_year { 400 } else { 0 };
    let rating_boost = (meta_text(meta, "imdbRating").parse::<f32>().unwrap_or(0.0) * 20.0) as i32;
    let rank_boost = meta_i64(meta, "rank")
        .map(|rank| (180 - rank as i32 * 12).max(0))
        .unwrap_or(0);
    Some(year_boost + rating_boost + rank_boost)
}

pub(crate) fn billboard_identity_key_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    Some(billboard_key_value(args.get("meta")?))
}

pub(crate) fn billboard_normalized_title(value: &str) -> String {
    normalized_billboard_title(value)
}

pub(crate) fn build_billboard_pool_json(
    enriched_json: &str,
    candidates_json: &str,
) -> Option<String> {
    let enriched: Vec<Value> = serde_json::from_str(enriched_json).ok()?;
    let candidates: Vec<Value> = serde_json::from_str(candidates_json).ok()?;

    let enriched_by_key: HashMap<String, Value> = enriched
        .iter()
        .map(|m| (billboard_key_value(m), m.clone()))
        .collect();

    // Editorial picks: prefer the enriched version, fall back to original when it has artwork.
    let editorial_raw: Vec<Value> = candidates
        .iter()
        .filter(|m| meta_text(m, "reason") == "EDITORIAL_SPOTLIGHT")
        .filter_map(|m| {
            let key = billboard_key_value(m);
            enriched_by_key.get(&key).cloned().or_else(|| {
                if has_backdrop_candidate(m) || !meta_text(m, "poster").is_empty() {
                    Some(m.clone())
                } else {
                    None
                }
            })
        })
        .collect();

    let mut editorial = editorial_raw
        .into_iter()
        .map(|item| {
            let score = score_candidate(&item, None);
            (score, item)
        })
        .collect::<Vec<_>>();
    editorial.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let editorial: Vec<Value> =
        distinct_by_title_key(editorial.into_iter().map(|(_, item)| item).collect())
            .into_iter()
            .take(3)
            .collect();

    // Ranked pool: merge enriched + candidates, deduplicate, filter, sort by score+visual.
    let combined: Vec<Value> = enriched.into_iter().chain(candidates).collect();
    let combined = distinct_by_title_key(distinct_by_billboard_key(combined));
    let mut ranked: Vec<(i32, Value)> = combined
        .into_iter()
        .filter(|m| has_backdrop_candidate(m) || !meta_text(m, "poster").is_empty())
        .map(|item| {
            let score = score_candidate(&item, None) + billboard_visual_score(&item);
            (score, item)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0));

    let series: Vec<Value> = ranked
        .iter()
        .filter(|(_, m)| meta_text(m, "type") == "series")
        .take(8)
        .map(|(_, m)| m.clone())
        .collect();
    let movies: Vec<Value> = ranked
        .iter()
        .filter(|(_, m)| meta_text(m, "type") == "movie")
        .take(3)
        .map(|(_, m)| m.clone())
        .collect();

    let preferred: Vec<Value> = distinct_by_title_key(distinct_by_billboard_key(
        editorial.into_iter().chain(series).chain(movies).collect(),
    ));

    let final_pool: Vec<Value> = if preferred.len() >= 10 {
        preferred.into_iter().take(10).collect()
    } else {
        let preferred_keys: HashSet<String> = preferred.iter().map(billboard_key_value).collect();
        let preferred_titles: HashSet<String> = preferred.iter().map(title_key_value).collect();
        let extras = ranked.into_iter().map(|(_, m)| m).filter(|m| {
            !preferred_keys.contains(&billboard_key_value(m))
                && !preferred_titles.contains(&title_key_value(m))
        });
        preferred.into_iter().chain(extras).take(10).collect()
    };

    serde_json::to_string(&final_pool).ok()
}

fn iso_date_part(date_str: &str) -> Option<&str> {
    let s = date_str.trim();
    let date_part = s.get(..10)?;
    let b = date_part.as_bytes();
    if b.get(4) == Some(&b'-') && b.get(7) == Some(&b'-') {
        Some(date_part)
    } else {
        None
    }
}

fn is_upcoming_date(date_str: &str, today_iso: &str) -> bool {
    iso_date_part(date_str).is_some_and(|d| d > today_iso)
}

pub(crate) fn normalize_home_catalog_items_json(
    items_json: &str,
    catalog_id: &str,
    genre: Option<&str>,
    today_iso: &str,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let assign_rank =
        genre.map(|g| g.is_empty()).unwrap_or(true) && RANKED_CATALOG_IDS.contains(&catalog_id);

    let mut rank: i64 = 0;
    let result: Vec<Value> = items
        .into_iter()
        .filter_map(|mut item| {
            let released = item.get("released").and_then(Value::as_str).unwrap_or("");
            if is_upcoming_date(released, today_iso) {
                return None;
            }
            if assign_rank {
                rank += 1;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("rank".to_string(), json!(rank));
                }
            }
            Some(item)
        })
        .collect();

    serde_json::to_string(&result).ok()
}
