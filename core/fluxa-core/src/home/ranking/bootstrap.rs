use super::folders::build_home_collection_shelves_json;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(crate) fn home_metadata_feed_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let feeds = request.get("feeds")?.as_array()?;
    let available_keys = feeds
        .iter()
        .filter_map(|feed| feed.get("key").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let ordered_keys = crate::catalog::identity::ordered_metadata_feed_keys(
        &serde_json::to_string(&available_keys).ok()?,
        &serde_json::to_string(request.get("order").unwrap_or(&Value::Null)).ok()?,
    )
    .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())?;
    let selected = request
        .get("selectedKeys")
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(Value::as_str)
                .filter(|key| available_keys.iter().any(|available| available == key))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
    // Match Settings/web semantics: an empty list means "all feeds enabled".
    // It can also happen when persisted keys no longer match refreshed add-ons;
    // in either case don't silently remove every Home catalog.
    let visible_keys = selected
        .filter(|keys| !keys.is_empty())
        .unwrap_or_else(|| available_keys.clone());
    let ordered = ordered_keys
        .iter()
        .filter(|key| visible_keys.contains(key))
        .filter_map(|key| {
            feeds
                .iter()
                .find(|feed| feed.get("key").and_then(Value::as_str) == Some(key))
        })
        .cloned()
        .collect::<Vec<_>>();
    serde_json::to_string(&ordered).ok()
}

fn hero_episode_plan_value(request: &Value) -> Option<Value> {
    let content_type = request.get("type").and_then(Value::as_str).unwrap_or("");
    if !matches!(content_type, "series" | "tv" | "show") {
        return None;
    }
    let videos = request.get("videos").and_then(Value::as_array)?;
    let regular_episodes = videos
        .iter()
        .filter(|video| {
            video
                .get("season")
                .and_then(Value::as_i64)
                .is_some_and(|season| season > 0)
                && video
                    .get("number")
                    .or_else(|| video.get("episode"))
                    .and_then(Value::as_i64)
                    .is_some_and(|number| number > 0)
        })
        .collect::<Vec<_>>();
    if regular_episodes.is_empty() {
        return None;
    }
    let last_video_id = request
        .get("lastVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty());
    let selected = last_video_id
        .and_then(|id| {
            regular_episodes
                .iter()
                .position(|video| video.get("id").and_then(Value::as_str) == Some(id))
        })
        .map(|index| {
            regular_episodes
                .get(index + 1)
                .copied()
                .unwrap_or(regular_episodes[0])
        })
        .unwrap_or(regular_episodes[0]);
    let season = selected.get("season").and_then(Value::as_i64)?;
    let number = selected
        .get("number")
        .or_else(|| selected.get("episode"))
        .and_then(Value::as_i64)?;
    Some(json!({
        "episode": {
            "id": selected.get("id").cloned().unwrap_or(Value::Null),
            "name": selected.get("name").or_else(|| selected.get("title")).cloned().unwrap_or(Value::Null),
            "season": season,
            "number": number,
        },
        "isContinue": last_video_id.is_some(),
    }))
}

pub(crate) fn home_hero_episode_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    serde_json::to_string(&hero_episode_plan_value(&request)?).ok()
}

pub(crate) fn home_hero_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    serde_json::to_string(&home_hero_plan(&request)).ok()
}

pub fn home_hero_plan(request: &Value) -> Value {
    let empty = json!({});
    let prefs = request
        .get("prefs")
        .filter(|p| p.is_object())
        .unwrap_or(&empty);
    let show_hero = prefs
        .get("showHeroSection")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut categories = request
        .get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|category| {
            !matches!(
                category.get("type").and_then(Value::as_str),
                Some("collection" | "collection_folder")
            )
        })
        .collect::<Vec<_>>();
    let id_of = |category: &Value| {
        category
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let hero_toggles = str_set(prefs.get("heroFeedToggles"));
    if !hero_toggles.is_empty()
        && categories
            .iter()
            .any(|category| id_of(category).is_some_and(|id| hero_toggles.contains(id.as_str())))
    {
        categories.retain(|category| {
            id_of(category).is_some_and(|id| hero_toggles.contains(id.as_str()))
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
        categories.sort_by_key(|category| {
            category
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| hero_order.get(id).copied())
                .unwrap_or(usize::MAX)
        });
    }
    categories.truncate(2);
    let billboard = categories
        .first()
        .and_then(|category| items_of(category).first());
    let mut seen = HashSet::new();
    let slides = billboard
        .into_iter()
        .chain(categories.iter().flat_map(|category| items_of(category)))
        .filter(|item| {
            (non_empty(item, "background") || non_empty(item, "poster"))
                && seen.insert(
                    item.get("id")
                        .or_else(|| item.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or(""),
                )
        })
        .take(8);
    let fetched_trailers = request.get("fetchedTrailers").and_then(Value::as_object);
    let fetched_logos = request.get("fetchedLogos").and_then(Value::as_object);
    let prepare = |item: &Value| {
        let mut item = item.clone();
        let id = item.get("id").and_then(Value::as_str).map(str::to_owned);
        let Some(fields) = item.as_object_mut() else {
            return item;
        };
        if !has_playable_trailer(fields.get("trailers"))
            && let Some(trailers) = id
                .as_deref()
                .and_then(|id| fetched_trailers?.get(id))
                .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
        {
            fields.insert("trailers".to_owned(), trailers.clone());
        }
        if !fields
            .get("logo")
            .and_then(Value::as_str)
            .is_some_and(|v| !v.is_empty())
            && let Some(logo) = id
                .as_deref()
                .and_then(|id| fetched_logos?.get(id))
                .filter(|value| value.as_str().is_some_and(|s| !s.is_empty()))
        {
            fields.insert("logo".to_owned(), logo.clone());
        }
        if let Some(shortened) = fields
            .get("description")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(crate::catalog::identity::shorten_synopsis)
        {
            fields.insert("description".to_owned(), Value::String(shortened));
        }
        if !fields.contains_key("heroEpisode")
            && let Some(plan) = hero_episode_plan_value(&json!({
                "type": fields.get("type"),
                "videos": fields.get("videos"),
            }))
        {
            fields.insert("heroEpisode".to_owned(), plan["episode"].clone());
        }
        item
    };
    let billboard = billboard.map(&prepare);
    let slides = slides.map(&prepare).collect::<Vec<_>>();
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
    let targets = |fetched: HashSet<&str>, missing: &dyn Fn(&Value) -> bool| {
        let mut seen = HashSet::new();
        billboard
            .iter()
            .chain(slides.iter())
            .filter(|item| {
                let id = item.get("id").and_then(Value::as_str).unwrap_or("");
                !id.is_empty() && missing(item) && !fetched.contains(id) && seen.insert(id)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let trailer_targets = if trailer_fetch_enabled {
        targets(str_set(request.get("fetchedIds")), &|item| {
            !has_playable_trailer(item.get("trailers"))
        })
    } else {
        Vec::new()
    };
    let logo_targets = targets(str_set(request.get("fetchedLogoIds")), &|item| {
        !non_empty(item, "logo")
    });
    json!({
        "billboard": billboard,
        "slides": slides,
        "trailerTargets": trailer_targets,
        "logoTargets": logo_targets,
        "showHero": show_hero,
        "autoplayTrailer": autoplay,
    })
}

fn items_of(category: &Value) -> &[Value] {
    category
        .get("items")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn str_set(value: Option<&Value>) -> HashSet<&str> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

fn non_empty(item: &Value, key: &str) -> bool {
    item.get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn has_playable_trailer(trailers: Option<&Value>) -> bool {
    trailers.and_then(Value::as_array).is_some_and(|trailers| {
        trailers.iter().any(|trailer| {
            trailer
                .get("url")
                .and_then(Value::as_str)
                .is_some_and(|url| url.contains("youtube.com") || url.contains("youtu.be"))
        })
    })
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
    let continue_watching: Value =
        crate::services::tracking::external_sync::merge_continue_watching_lists_json(
            &Value::Array(local).to_string(),
            &Value::Array(external).to_string(),
            &progress.to_string(),
            prefs.get("syncCwSourceOfTruth").and_then(Value::as_str),
            prefs.get("syncCwRanking").and_then(Value::as_str),
        )
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_else(|| json!([]));
    let addons_json = Value::Array(addons.clone()).to_string();
    let mut feeds: Vec<Value> =
        crate::catalog::search::build_metadata_feed_options_json(&addons_json)
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default();
    for feed in &mut feeds {
        if let Some(genre) =
            crate::catalog::search::resolve_feed_option_genre_json(&feed.to_string(), &addons_json)
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
        crate::catalog::identity::effective_metadata_feed_selection_json(
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
