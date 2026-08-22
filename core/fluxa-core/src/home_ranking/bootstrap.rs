use super::folders::build_home_collection_shelves_json;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(crate) fn home_hero_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let prefs = request.get("prefs").cloned().unwrap_or_else(|| json!({}));
    let safe: Value = crate::profile_prefs::profile_safe_prefs_json(&prefs.to_string())
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_else(|| json!({}));
    let categories = request
        .get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|category| {
            let mut category = category.clone();
            if !category.get("items").is_some_and(Value::is_array)
                && let Some(fields) = category.as_object_mut()
            {
                fields.insert("items".to_string(), json!([]));
            }
            category
        })
        .collect::<Vec<_>>();
    let mut content_categories = categories
        .iter()
        .filter(|category| {
            !matches!(
                category.get("type").and_then(Value::as_str),
                Some("collection" | "collection_folder")
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let hero_toggles = prefs
        .get("heroFeedToggles")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    if !hero_toggles.is_empty() {
        content_categories.retain(|category| {
            category
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| hero_toggles.contains(id))
        });
    }
    let hero_order = prefs
        .get("heroFeedOrder")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<HashMap<_, _>>();
    if !hero_order.is_empty() {
        content_categories.sort_by_key(|category| {
            category
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| hero_order.get(id).copied())
                .unwrap_or(usize::MAX)
        });
    }
    if hero_toggles.is_empty() {
        content_categories.truncate(2);
    } else if content_categories.len() > 2 {
        content_categories.truncate(2);
    }
    let billboard = content_categories
        .first()
        .and_then(|category| category.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .or_else(|| {
            request
                .get("billboard")
                .filter(|value| !value.is_null())
                .cloned()
        });
    let mut seen = HashSet::new();
    let mut slides = billboard
        .iter()
        .chain(content_categories.iter().flat_map(|category| {
            category
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        }))
        .filter(|item| {
            (item
                .get("background")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
                || item
                    .get("poster")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty()))
                && seen.insert(
                    item.get("id")
                        .or_else(|| item.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                )
        })
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let fetched_trailers = request.get("fetchedTrailers").and_then(Value::as_object);
    let has_playable = |item: &Value| {
        item.get("trailers")
            .and_then(Value::as_array)
            .is_some_and(|trailers| {
                trailers.iter().any(|trailer| {
                    trailer
                        .get("url")
                        .and_then(Value::as_str)
                        .is_some_and(|url| url.contains("youtube.com") || url.contains("youtu.be"))
                })
            })
    };
    let merge_trailers = |mut item: Value| {
        if !has_playable(&item)
            && let Some(trailers) = item
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| fetched_trailers.and_then(|values| values.get(id)))
                .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
            && let Some(fields) = item.as_object_mut()
        {
            fields.insert("trailers".to_string(), trailers.clone());
        }
        item
    };
    let fetched_logos = request.get("fetchedLogos").and_then(Value::as_object);
    let has_logo = |item: &Value| {
        item.get("logo")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
    };
    let merge_logos = |mut item: Value| {
        if !has_logo(&item)
            && let Some(logo) = item
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| fetched_logos.and_then(|values| values.get(id)))
                .filter(|value| value.as_str().is_some_and(|s| !s.is_empty()))
            && let Some(fields) = item.as_object_mut()
        {
            fields.insert("logo".to_string(), logo.clone());
        }
        item
    };
    let shorten_description = |mut item: Value| {
        let shortened = item
            .get("description")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(crate::content_identity::shorten_synopsis);
        if let Some(shortened) = shortened
            && let Some(fields) = item.as_object_mut()
        {
            fields.insert("description".to_string(), Value::String(shortened));
        }
        item
    };
    let billboard = billboard
        .map(&merge_trailers)
        .map(&merge_logos)
        .map(&shorten_description);
    slides = slides
        .into_iter()
        .map(&merge_trailers)
        .map(&merge_logos)
        .map(&shorten_description)
        .collect();
    let autoplay = prefs
        .get("homeHeroAutoplayTrailer")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let trailer_fetch_enabled = autoplay
        && prefs
            .get("tmdbTrailersEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        && prefs
            .get("tmdbApiKey")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
    let fetched_ids = request
        .get("fetchedIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let mut target_seen = HashSet::new();
    let trailer_targets = billboard
        .iter()
        .chain(slides.iter())
        .filter(|item| {
            let id = item.get("id").and_then(Value::as_str).unwrap_or("");
            trailer_fetch_enabled
                && !id.is_empty()
                && !has_playable(item)
                && !fetched_ids.contains(id)
                && target_seen.insert(id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let fetched_logo_ids = request
        .get("fetchedLogoIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let mut logo_target_seen = HashSet::new();
    let logo_targets = billboard
        .iter()
        .chain(slides.iter())
        .filter(|item| {
            let id = item.get("id").and_then(Value::as_str).unwrap_or("");
            !id.is_empty()
                && !has_logo(item)
                && !fetched_logo_ids.contains(id)
                && logo_target_seen.insert(id)
        })
        .cloned()
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({
        "categories": categories, "contentCategories": content_categories, "billboard": billboard, "slides": slides,
        "trailerTargets": trailer_targets, "logoTargets": logo_targets,
        "showHero": safe.get("showHeroSection").and_then(Value::as_bool).unwrap_or(true), "autoplayTrailer": autoplay
    })).ok()
}

#[expect(
    dead_code,
    reason = "kept as a platform-specific FFI planning entry point"
)]
pub(crate) fn home_bootstrap_preparation_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let profile = request.get("profile").cloned().unwrap_or_else(|| json!({}));
    let prefs = request.get("prefs").cloned().unwrap_or_else(|| json!({}));
    let library = request.get("library").cloned().unwrap_or_else(|| json!({}));
    let disabled = profile
        .pointer("/addonSettings/disabledLocalAddons")
        .or_else(|| profile.get("disabledLocalAddons"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let mut addons = request
        .get("addons")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|addon| {
            let key = addon
                .get("transportUrl")
                .and_then(Value::as_str)
                .or_else(|| addon.pointer("/manifest/id").and_then(Value::as_str))
                .unwrap_or("");
            !disabled.contains(key)
        })
        .collect::<Vec<_>>();
    if let Some(builtin) = request
        .get("builtinAddon")
        .filter(|value| value.is_object())
    {
        if prefs
            .get("tmdbPreferOverAddons")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            addons.insert(0, builtin.clone());
        } else {
            addons.push(builtin.clone());
        }
    }
    let local = library
        .get("continueWatching")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let external = library
        .get("externalContinueWatching")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let progress = library
        .get("progress")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let continue_watching: Value = crate::external_sync::merge_continue_watching_lists_json(
        &Value::Array(local).to_string(),
        &Value::Array(external).to_string(),
        &progress.to_string(),
        prefs.get("syncCwSourceOfTruth").and_then(Value::as_str),
        prefs.get("syncCwRanking").and_then(Value::as_str),
    )
    .and_then(|value| serde_json::from_str(&value).ok())
    .unwrap_or_else(|| json!([]));
    let addons_json = Value::Array(addons.clone()).to_string();
    let mut feeds: Vec<Value> = crate::search_plan::build_metadata_feed_options_json(&addons_json)
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_default();
    for feed in &mut feeds {
        if let Some(genre) =
            crate::search_plan::resolve_feed_option_genre_json(&feed.to_string(), &addons_json)
                .and_then(|value| serde_json::from_str(&value).ok())
        {
            feed["genre"] = genre;
        }
    }
    let available = feeds
        .iter()
        .filter_map(|feed| feed.get("key").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let selected = prefs.get("homeFeedToggles").and_then(Value::as_array);
    let effective = if selected.is_some_and(|values| !values.is_empty()) {
        crate::content_identity::effective_metadata_feed_selection_json(
            &Value::Array(selected.cloned().unwrap_or_default()).to_string(),
            &Value::Array(available.iter().map(|value| json!(value)).collect()).to_string(),
        )
        .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
        .unwrap_or_else(|| available.clone())
    } else {
        available
    };
    let visible_feeds = feeds
        .iter()
        .filter(|feed| {
            feed.get("key")
                .and_then(Value::as_str)
                .is_some_and(|key| effective.iter().any(|value| value == key))
        })
        .cloned()
        .collect::<Vec<_>>();
    let shelves: Value = build_home_collection_shelves_json(&profile.to_string(), &addons_json)
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_else(
            || json!({"pinnedShelves": [], "regularShelves": [], "hiddenFolderCategories": []}),
        );
    serde_json::to_string(&json!({"addons": addons, "continueWatching": continue_watching, "metadataFeeds": feeds, "visibleFeeds": visible_feeds, "shelves": shelves, "feedConcurrency": 6})).ok()
}

#[expect(
    dead_code,
    reason = "kept as a platform-specific FFI planning entry point"
)]
pub(crate) fn home_bootstrap_completion_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let preparation = request.get("preparation")?;
    let categories = request.get("feedResults").and_then(Value::as_array).into_iter().flatten().filter_map(|result| {
        let feed = result.get("feed")?;
        let items = result.get("items").and_then(Value::as_array)?;
        if items.is_empty() { return None; }
        let label = feed.get("label").and_then(Value::as_str).unwrap_or("");
        Some(json!({
            "id": feed.get("key"), "name": feed.get("homeTitle").and_then(Value::as_str).unwrap_or(label),
            "semanticName": feed.get("homeTitle").and_then(Value::as_str).unwrap_or(label), "type": feed.get("type"), "items": items,
            "addonName": label.split(" - ").next().unwrap_or(label), "transportUrl": feed.get("transportUrl"), "catalogId": feed.get("id")
        }))
    }).collect::<Vec<_>>();
    let shelves = preparation
        .get("shelves")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut all = shelves
        .get("pinnedShelves")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    all.extend(categories.iter().cloned());
    all.extend(
        shelves
            .get("regularShelves")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .cloned(),
    );
    all.extend(
        shelves
            .get("hiddenFolderCategories")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .cloned(),
    );
    let billboard = categories
        .first()
        .and_then(|category| category.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned();
    serde_json::to_string(&json!({"categories": all, "continueWatching": preparation.get("continueWatching"), "metadataFeeds": preparation.get("metadataFeeds"), "billboard": billboard})).ok()
}
