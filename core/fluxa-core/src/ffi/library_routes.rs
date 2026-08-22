use super::*;

pub(super) fn route_library_state(method: &str, args_json: &str) -> Outcome {
    match method {
        "playbackProgressItem" => {
            let args = object(args_json)?;
            let time_offset = field(&args, "timeOffset")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "timeOffset must be a number"))?;
            let duration = field(&args, "duration")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "duration must be a number"))?;
            opt_json(library_state::playback_progress_item_json(
                field_str(&args, "metaJson")?,
                time_offset,
                duration,
                field_str(&args, "nowUtc")?,
            ))
        }
        "clearPlaybackProgressItem" => opt_json(library_state::clear_playback_progress_item_json(
            &arg_str(args_json, "metaJson")?,
        )),
        "clearPlaybackProgressPlan" => {
            opt_json(library_state::clear_playback_progress_plan_json(args_json))
        }
        "watchedStateItems" => {
            let args = object(args_json)?;
            let watched = field(&args, "watched")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "watched must be bool"))?;
            let watched_at = args.get("watchedAt").and_then(Value::as_str);
            opt_json(library_state::watched_state_items_json(
                field_str(&args, "metaJson")?,
                field_str(&args, "episodesJson")?,
                watched,
                watched_at,
            ))
        }
        "filterHomeContinueWatching" => {
            let args = object(args_json)?;
            let trakt_watched_json = args
                .get("traktWatchedJson")
                .and_then(Value::as_str)
                .unwrap_or("");
            opt_json(library_state::filter_home_continue_watching_json(
                field_str(&args, "itemsJson")?,
                trakt_watched_json,
            ))
        }
        "continueWatchingSourcePlan" => {
            opt_json(library_state::continue_watching_source_plan_json(args_json))
        }
        "watchedVideoIds" => {
            let args = object(args_json)?;
            opt_json(library_state::watched_video_ids_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "imdbId")?,
            ))
        }
        "libraryDocumentViews" => opt_json(Some(library_persistence::document_views_json(
            &arg_str(args_json, "documentJson")?,
        ))),
        "isEpisodeReleased" => {
            let args = object(args_json)?;
            let video: Value =
                serde_json::from_str(field_str(&args, "videoJson")?).map_err(|e| {
                    fail(
                        ErrorKind::InvalidArgs,
                        format!("videoJson is not valid JSON: {e}"),
                    )
                })?;
            let now_ms = field_u64(&args, "nowMs")? as i64;
            Ok(json!(library_state::is_episode_released(&video, now_ms)))
        }
        "curateHomeItems" => opt_json(home_ranking::curate_home_items_json(&arg_str(
            args_json,
            "categoryJson",
        )?)),
        "homeOverlapRatio" => {
            let args = object(args_json)?;
            Ok(json!(home_ranking::home_overlap_ratio_json(
                field_str(&args, "firstJson")?,
                field_str(&args, "secondJson")?,
            )))
        }
        "homePersonalizationScore" => {
            let args = object(args_json)?;
            Ok(json!(home_ranking::home_personalization_score_json(
                field_str(&args, "categoryJson")?,
                field_str(&args, "preferredGenresJson")?,
                field_str(&args, "preferredTypesJson")?,
                field_str(&args, "priorityLabelsJson")?,
            )))
        }
        "prioritizeHomeRows" => {
            let args = object(args_json)?;
            opt_json(home_ranking::home_prioritize_rows_json(
                field_str(&args, "categoriesJson")?,
                field_str(&args, "preferredOrderLabelsJson")?,
                field_str(&args, "preferredGenresJson")?,
                field_str(&args, "preferredTypesJson")?,
                field_str(&args, "priorityLabelsJson")?,
            ))
        }
        "optimizeHomeRows" => opt_json(home_ranking::optimize_home_rows_json(args_json)),
        "buildBillboardPool" => {
            let args = object(args_json)?;
            opt_json(home_ranking::build_billboard_pool_json(
                field_str(&args, "enrichedJson")?,
                field_str(&args, "candidatesJson")?,
            ))
        }
        "normalizeHomeCatalogItems" => {
            let args = object(args_json)?;
            let genre = args.get("genre").and_then(Value::as_str);
            opt_json(home_ranking::normalize_home_catalog_items_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "catalogId")?,
                genre,
                field_str(&args, "todayIso")?,
            ))
        }
        // args_json IS the items/item/doc JSON for single-arg methods
        "libraryContinueWatchingItems" => opt_json(
            library_state::library_continue_watching_items_json(args_json),
        ),
        "libraryWatchlistItems" => opt_json(library_state::library_watchlist_items_json(args_json)),
        "normalizeLibraryDocument" => {
            into_json(library_state::normalize_library_document_json(args_json))
        }
        "isUpNextContinueWatchingItem" => Ok(json!(
            library_state::is_up_next_continue_watching_item_json(args_json)
        )),
        "buildContinueWatchingFromProgress" => opt_json(
            library_state::build_continue_watching_from_progress_json(args_json),
        ),
        "rememberLastWatchedEpisodes" => {
            let args = object(args_json)?;
            into_json(library_state::remember_last_watched_episodes_json(
                field_str(&args, "libJson")?,
                field_str(&args, "watchedIdsJson")?,
            ))
        }
        "computeContinueWatchingBadges" => {
            let args = object(args_json)?;
            let now_ms = field(&args, "nowMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?;
            opt_json(library_state::compute_continue_watching_badges_json(
                field_str(&args, "candidatesJson")?,
                field_str(&args, "videosBySeriesJson")?,
                field_str(&args, "lastWatchedJson")?,
                now_ms,
            ))
        }
        "resolveNextEpisode" => {
            let args = object(args_json)?;
            opt_json(library_state::resolve_next_episode_json(
                &field(&args, "videos")?.to_string(),
                field(&args, "currentSeason")?.as_i64().ok_or_else(|| {
                    fail(ErrorKind::InvalidArgs, "currentSeason must be a number")
                })?,
                field(&args, "currentEpisode")?.as_i64().ok_or_else(|| {
                    fail(ErrorKind::InvalidArgs, "currentEpisode must be a number")
                })?,
                field(&args, "nowMs")?
                    .as_i64()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?,
                field(&args, "releasedOnly")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "releasedOnly must be bool"))?,
            ))
        }
        "resolveNextAfterWatched" => {
            opt_json(library_state::resolve_next_after_watched_json(args_json))
        }
        "nextProgressInfoPlan" => opt_json(library_state::next_progress_info_plan_json(args_json)),
        "formatEpisodeLine" => {
            let args = object(args_json)?;
            Ok(Value::String(library_state::format_episode_line_json(
                args.get("lastEpisodeName").and_then(Value::as_str),
                args.get("lastEpisodeSeason").and_then(Value::as_i64),
                args.get("lastEpisodeNumber").and_then(Value::as_i64),
                args.get("lastVideoId").and_then(Value::as_str),
            )))
        }
        "selectContinueWatchingArtwork" => {
            let args = object(args_json)?;
            Ok(json!(library_state::select_continue_watching_artwork_json(
                &field(&args, "item")?.to_string(),
                field_str(&args, "artworkPreference")?,
                field(&args, "isHorizontal")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isHorizontal must be bool"))?,
            )))
        }
        "continueWatchingCardFields" => {
            let args = object(args_json)?;
            opt_json(library_state::continue_watching_card_fields_json(
                &field(&args, "items")?.to_string(),
                field_str(&args, "artworkPreference")?,
                field(&args, "isHorizontal")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isHorizontal must be bool"))?,
            ))
        }
        "buildHomeCollectionShelves" => {
            let args = object(args_json)?;
            opt_json(home_ranking::build_home_collection_shelves_json(
                field_str(&args, "profileJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }
        "folderPageState" => opt_json(home_ranking::folder_page_state_json(args_json)),
        "folderSourcePagePlan" => opt_json(home_ranking::folder_source_page_plan_json(args_json)),
        "homeHeroPlan" => opt_json(home_ranking::home_hero_plan_json(args_json)),
        "homeBillboardCandidateScore" => Ok(json!(home_ranking::billboard_candidate_score_json(
            args_json
        ))),
        "homeBillboardVisualScore" => {
            Ok(json!(home_ranking::billboard_visual_score_json(args_json)))
        }
        "homeBillboardHasBackdrop" => {
            Ok(json!(home_ranking::billboard_has_backdrop_json(args_json)))
        }
        "homeBillboardEditorialMatchScore" => Ok(json!(
            home_ranking::billboard_editorial_match_score_json(args_json)
        )),
        "homeBillboardIdentityKey" => {
            Ok(json!(home_ranking::billboard_identity_key_json(args_json)))
        }
        "homeBillboardNormalizedTitle" => Ok(Value::String(
            home_ranking::billboard_normalized_title(&arg_str(args_json, "value")?),
        )),
        "mergeFolderSources" => opt_json(home_ranking::merge_folder_sources_json(args_json)),
        "watchedMapDiff" => {
            let args = object(args_json)?;
            opt_json(library_state::watched_map_diff_json(
                field_str(&args, "beforeJson")?,
                field_str(&args, "afterJson")?,
            ))
        }
        "valueMapDiff" => {
            let args = object(args_json)?;
            opt_json(library_state::value_map_diff_json(
                field_str(&args, "beforeJson")?,
                field_str(&args, "afterJson")?,
            ))
        }
        "itemListDiff" => {
            let args = object(args_json)?;
            opt_json(library_state::item_list_diff_json(
                field_str(&args, "beforeJson")?,
                field_str(&args, "afterJson")?,
            ))
        }
        "itemListNewEntries" => {
            let args = object(args_json)?;
            opt_json(library_state::item_list_new_entries_json(
                field_str(&args, "beforeJson")?,
                field_str(&args, "afterJson")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
