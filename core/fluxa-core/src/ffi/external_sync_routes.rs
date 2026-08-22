use super::*;

pub(super) fn route_external_sync_trakt(method: &str, args_json: &str) -> Outcome {
    match method {
        "externalSyncResponseAction" => {
            let args = object(args_json)?;
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?;
            Ok(Value::String(
                external_sync::external_sync_response_action(
                    field_str(&args, "provider")?,
                    status_code,
                )
                .to_string(),
            ))
        }
        "externalSyncRefreshRetryAction" => {
            let args = object(args_json)?;
            Ok(Value::String(
                external_sync::external_sync_refresh_retry_action(
                    args.get("statusCode").and_then(Value::as_i64),
                )
                .to_string(),
            ))
        }
        "providerCalendarItems" => opt_json(external_sync::provider_calendar_items_json(args_json)),
        "providerPaginationPlan" => {
            opt_json(external_sync::provider_pagination_plan_json(args_json))
        }
        "stremioLibraryMutationPlan" => {
            opt_json(external_sync::stremio_library_mutation_plan_json(args_json))
        }
        "traktHasClient" => Ok(json!(external_sync::trakt_has_client(&arg_str(
            args_json, "apiKey",
        )?))),
        "traktBearer" => Ok(Value::String(external_sync::trakt_bearer(&arg_str(
            args_json, "token",
        )?))),
        "traktScrobbleUrl" => opt_str(external_sync::trakt_scrobble_url(&arg_str(
            args_json, "action",
        )?)),
        "traktPlaybackUrl" => {
            let args = object(args_json)?;
            let content_type = args.get("contentType").and_then(Value::as_str);
            opt_str(external_sync::trakt_playback_url(content_type))
        }
        "traktTokenExpiresAt" => {
            let args = object(args_json)?;
            let created_at_seconds = field(&args, "createdAtSeconds")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "createdAtSeconds must be a number"))?;
            let expires_in_seconds = field(&args, "expiresInSeconds")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "expiresInSeconds must be a number"))?;
            Ok(json!(external_sync::trakt_token_expires_at(
                created_at_seconds,
                expires_in_seconds,
            )))
        }
        "traktContentIdFromIds" => opt_str(external_sync::trakt_content_id_from_ids_json(
            &arg_str(args_json, "idsJson")?,
        )),
        "traktSyncItemToMeta" => opt_json(external_sync::trakt_sync_item_to_meta_json(args_json)),
        "traktPlaybackDeleteIds" => {
            opt_json(external_sync::trakt_playback_delete_ids_json(args_json))
        }
        "traktIdsFromContentId" => opt_json(external_sync::trakt_ids_from_content_id_json(
            &arg_str(args_json, "rawId")?,
        )),
        "traktEpisodeLocator" => opt_json(external_sync::trakt_episode_locator_json(&arg_str(
            args_json, "videoId",
        )?)),
        "traktShowIdFromEpisodeId" => Ok(Value::String(
            external_sync::trakt_show_id_from_episode_id(&arg_str(args_json, "videoId")?),
        )),
        "traktScrobbleMediaId" => {
            let args = object(args_json)?;
            let video_id = args.get("videoId").and_then(Value::as_str);
            Ok(Value::String(external_sync::trakt_scrobble_media_id(
                field_str(&args, "parentId")?,
                video_id,
                field_str(&args, "mediaType")?,
            )))
        }
        "traktOAuthErrorCode" => opt_str(external_sync::trakt_oauth_error_code(&arg_str(
            args_json, "body",
        )?)),
        "traktHistoryRequest" => {
            let args = object(args_json)?;
            opt_json(external_sync::trakt_history_request_json(
                field_str(&args, "metaJson")?,
                field_str(&args, "episodesJson")?,
            ))
        }
        // args_json IS the items array for single-array-arg methods
        "traktPlaybackItemsToLibrary" => opt_json(
            external_sync::trakt_playback_items_to_library_json(args_json),
        ),
        "traktWatchedShowsToItems" => {
            opt_json(external_sync::trakt_watched_shows_to_items_json(args_json))
        }
        "traktWatchlistToItems" => {
            let args = object(args_json)?;
            opt_json(external_sync::trakt_watchlist_to_items_json(
                field_str(&args, "moviesJson")?,
                field_str(&args, "showsJson")?,
            ))
        }
        "stremioWatchlistToItems" => {
            opt_json(external_sync::stremio_watchlist_to_items_json(args_json))
        }
        "stremioWatchedToIds" => opt_json(external_sync::stremio_watched_to_ids_json(args_json)),
        "traktWatchedToIds" => {
            let args = object(args_json)?;
            opt_json(external_sync::trakt_watched_to_ids_json(
                field_str(&args, "moviesJson")?,
                field_str(&args, "showsJson")?,
            ))
        }
        "mergeExternalWatchlist" => {
            let args = object(args_json)?;
            into_json(external_sync::merge_external_watchlist_json(
                field_str(&args, "localJson")?,
                field_str(&args, "externalJson")?,
            ))
        }
        "mergeExternalWatched" => {
            let args = object(args_json)?;
            into_json(external_sync::merge_external_watched_json(
                field_str(&args, "localJson")?,
                field_str(&args, "externalJson")?,
            ))
        }
        "pushPlan" => opt_json(external_sync::push_plan_json(args_json)),
        "importApplyPlan" => opt_json(external_sync::import_apply_plan_json(args_json)),
        "mergeContinueWatchingLists" => {
            let args = object(args_json)?;
            opt_json(external_sync::merge_continue_watching_lists_json(
                field_str(&args, "localJson")?,
                field_str(&args, "externalJson")?,
                field_str(&args, "progressJson")?,
                args.get("sourceOfTruth").and_then(Value::as_str),
                args.get("rankingMode").and_then(Value::as_str),
            ))
        }
        "mergeWatchlistTimestamped" => {
            let args = object(args_json)?;
            into_json(external_sync::merge_watchlist_timestamped_json(
                &field(&args, "local")?.to_string(),
                &field(&args, "remote")?.to_string(),
            ))
        }
        "mergeWatchedTimestamped" => {
            let args = object(args_json)?;
            into_json(external_sync::merge_watched_timestamped_json(
                &field(&args, "local")?.to_string(),
                &field(&args, "remote")?.to_string(),
            ))
        }
        "traktScrobblePlan" => {
            let args = object(args_json)?;
            let season = args.get("season").and_then(Value::as_i64);
            let ep_number = args.get("epNumber").and_then(Value::as_i64);
            let time_pos = field(&args, "timePosSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "timePosSec must be a number"))?;
            let duration = field(&args, "durationSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "durationSec must be a number"))?;
            let ids_json = content_identity::build_trakt_ids_json(field_str(&args, "videoId")?)
                .ok_or_else(|| fail(ErrorKind::NotFound, "could not build trakt ids"))?;
            opt_json(player_scrobble::trakt_scrobble_plan_json(
                &ids_json,
                field(&args, "isEpisode")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isEpisode must be bool"))?,
                season,
                ep_number,
                time_pos,
                duration,
                args.get("action").and_then(Value::as_str),
            ))
        }
        "replaceExternalContinueWatching" => {
            let args = object(args_json)?;
            let provider = args.get("provider").and_then(Value::as_str);
            into_json(external_sync::replace_external_continue_watching_json(
                field_str(&args, "existingJson")?,
                provider,
                field_str(&args, "itemsJson")?,
                args.get("sourceOfTruth").and_then(Value::as_str),
                args.get("rankingMode").and_then(Value::as_str),
                args.get("continueWatchingDays").and_then(Value::as_i64),
            ))
        }
        "promoteExternalProgressPlan" => opt_json(
            external_sync::promote_external_progress_plan_json(args_json),
        ),
        "externalProviderActionPlan" => {
            opt_json(external_sync::external_provider_action_plan_json(args_json))
        }
        "traktPlaybackItemsDedup" => {
            opt_json(external_sync::trakt_playback_items_dedup_json(args_json))
        }
        "traktUpNextToItems" => opt_json(external_sync::trakt_up_next_to_items_json(args_json)),
        "traktMarkWatchedBody" => opt_json(external_sync::trakt_mark_watched_body_json(args_json)),
        "traktRelatedLookupSlug" => {
            let args = object(args_json)?;
            opt_json(external_sync::trakt_related_lookup_slug(
                field_str(&args, "lookupJson")?,
                field_str(&args, "wantType")?,
            ))
        }
        "traktRelatedItemsToMetas" => {
            let args = object(args_json)?;
            opt_json(external_sync::trakt_related_items_to_metas_json(
                field_str(&args, "relatedJson")?,
                field_str(&args, "contentType")?,
            ))
        }
        "traktCommentsRequest" => opt_json(external_sync::trakt_comments_request_json(args_json)),
        "traktActivityDiff" => opt_json(external_sync::trakt_activity_diff_json(args_json)),

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_external_sync_simkl(method: &str, args_json: &str) -> Outcome {
    match method {
        "simklHistoryRequest" => opt_json(external_sync::simkl_history_request_json(args_json)),
        "simklWatchlistRequest" => opt_json(external_sync::simkl_watchlist_request_json(
            args_json, false,
        )),
        "simklWatchlistRemovalRequest" => {
            opt_json(external_sync::simkl_watchlist_request_json(args_json, true))
        }
        "simklMarkWatchedBody" => opt_json(external_sync::simkl_mark_watched_body_json(args_json)),
        "simklPlaybackDeleteIds" => {
            opt_json(external_sync::simkl_playback_delete_ids_json(args_json))
        }
        "simklPlaybackItemToContinueMeta" => opt_json(
            external_sync::simkl_playback_item_to_continue_meta_json(args_json),
        ),
        "simklWatchlistBody" => opt_json(external_sync::simkl_watchlist_body_json(args_json)),
        "simklWatchingToItems" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_watching_to_items_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklWatchlistToItems" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_watchlist_to_items_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklMergePlaybackProgress" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_merge_playback_progress_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "playbackJson")?,
            ))
        }
        "simklWatchedToIds" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_watched_to_ids_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklScrobbleBody" => {
            let args = object(args_json)?;
            let season = field(&args, "season")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "season must be a number"))?;
            let ep_number = field(&args, "epNumber")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "epNumber must be a number"))?;
            let time_pos = field(&args, "timePosSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "timePosSec must be a number"))?;
            let duration = field(&args, "durationSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "durationSec must be a number"))?;
            opt_json(player_scrobble::simkl_scrobble_body_json(
                field_str(&args, "idsJson")?,
                field(&args, "isEpisode")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isEpisode must be bool"))?,
                season,
                ep_number,
                time_pos,
                duration,
            ))
        }
        "simklMatchEpisode" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_match_episode_json(
                field_str(&args, "episodesJson")?,
                field_str(&args, "targetJson")?,
            ))
        }
        "simklLookupIdForType" => {
            let args = object(args_json)?;
            match external_sync::simkl_lookup_id_for_type(
                field_str(&args, "lookupJson")?,
                field_str(&args, "wantType")?,
            ) {
                Some(id) => Ok(json!(id)),
                None => Ok(Value::Null),
            }
        }
        "simklRecommendationCandidates" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_recommendation_candidates_json(
                field_str(&args, "detailJson")?,
            ))
        }
        "simklRecommendationToMeta" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_recommendation_to_meta_json(
                field_str(&args, "recJson")?,
                field_str(&args, "resolvedImdb")?,
            ))
        }
        "simklResourceSyncPlan" => {
            opt_json(external_sync::simkl_resource_sync_plan_json(args_json))
        }
        "simklMergeDelta" => {
            let args = object(args_json)?;
            opt_json(external_sync::simkl_merge_delta_json(
                field_str(&args, "previousJson")?,
                field_str(&args, "changesJson")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_external_sync_anilist(method: &str, args_json: &str) -> Outcome {
    match method {
        "anilistEntriesToSync" => {
            let args = object(args_json)?;
            let now_ms = field(&args, "nowMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?;
            let entries = field(&args, "entries")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "entries must be an array"))?;
            let categories: Option<Vec<&str>> = args
                .get("categories")
                .filter(|v| !v.is_null())
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).collect());
            let dry_run = args.get("dryRun").and_then(Value::as_bool).unwrap_or(false);
            Ok(external_sync::anilist_entries_to_sync(
                entries,
                now_ms,
                categories.as_deref(),
                dry_run,
            ))
        }
        "mergeLibraryItemsById" => {
            let args = object(args_json)?;
            let local = field(&args, "local")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "local must be an array"))?;
            let incoming = field(&args, "incoming")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "incoming must be an array"))?;
            Ok(external_sync::merge_library_items_by_id(local, incoming))
        }
        "anilistGraphqlQueries" => Ok(Value::String(external_sync::anilist_graphql_queries_json())),
        "anilistSaveMediaListEntryVariables" => {
            let args = object(args_json)?;
            let progress = args.get("progress").and_then(Value::as_i64);
            opt_json(external_sync::anilist_save_media_list_entry_variables_json(
                field_str(&args, "contentId")?,
                field_str(&args, "status")?,
                progress,
            ))
        }
        // args_json IS the meta object
        "extractAnilistIdFromLinks" => Ok(json!(external_sync::extract_anilist_id_from_links(
            &object(args_json)?
        ))),
        "anilistSearchBestMatch" => {
            opt_json(external_sync::anilist_search_best_match_json(args_json))
        }
        "anilistMediaListStatus" => {
            let args = object(args_json)?;
            let total_episodes = field(&args, "totalEpisodes")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "totalEpisodes must be a number"))?;
            let progress_episode = field(&args, "progressEpisode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "progressEpisode must be a number"))?;
            Ok(json!(external_sync::anilist_media_list_status(
                total_episodes,
                progress_episode
            )))
        }

        _ => Err(unknown_method()),
    }
}
