use crate::ffi::*;
use crate::services;
use crate::{catalog, player};
use serde_json::{Value, json};

pub(crate) fn route_trakt(method: &str, args_json: &str) -> Outcome {
    match method {
        "traktHasClient" => Ok(json!(services::trakt::trakt_has_client(&arg_str(
            args_json, "apiKey",
        )?))),
        "traktBearer" => Ok(Value::String(services::trakt::trakt_bearer(&arg_str(
            args_json, "token",
        )?))),
        "traktScrobbleUrl" => opt_str(services::trakt::trakt_scrobble_url(&arg_str(
            args_json, "action",
        )?)),
        "traktPlaybackUrl" => {
            let args = object(args_json)?;
            let content_type = args.get("contentType").and_then(Value::as_str);
            opt_str(services::trakt::trakt_playback_url(content_type))
        }
        "traktListReference" => opt_str(services::trakt::trakt_list_reference(&arg_str(
            args_json, "input",
        )?)),
        "traktTokenExpiresAt" => {
            let args = object(args_json)?;
            let created_at_seconds = field(&args, "createdAtSeconds")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "createdAtSeconds must be a number"))?;
            let expires_in_seconds = field(&args, "expiresInSeconds")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "expiresInSeconds must be a number"))?;
            Ok(json!(services::trakt::trakt_token_expires_at(
                created_at_seconds,
                expires_in_seconds,
            )))
        }
        "traktContentIdFromIds" => opt_str(services::trakt::trakt_content_id_from_ids_json(
            &arg_str(args_json, "idsJson")?,
        )),
        "traktSyncItemToMeta" => opt_json(services::trakt::trakt_sync_item_to_meta_json(args_json)),
        "traktSyncItemContentType" => opt_str(services::trakt::trakt_sync_item_content_type_json(
            args_json,
        )),
        "traktPlaybackDeleteIds" => {
            opt_json(services::trakt::trakt_playback_delete_ids_json(args_json))
        }
        "traktIdsFromContentId" => opt_json(services::trakt::trakt_ids_from_content_id_json(
            &arg_str(args_json, "rawId")?,
        )),
        "traktEpisodeLocator" => opt_json(services::trakt::trakt_episode_locator_json(&arg_str(
            args_json, "videoId",
        )?)),
        "traktShowIdFromEpisodeId" => Ok(Value::String(
            services::trakt::trakt_show_id_from_episode_id(&arg_str(args_json, "videoId")?),
        )),
        "traktScrobbleMediaId" => {
            let args = object(args_json)?;
            let video_id = args.get("videoId").and_then(Value::as_str);
            Ok(Value::String(services::trakt::trakt_scrobble_media_id(
                field_str(&args, "parentId")?,
                video_id,
                field_str(&args, "mediaType")?,
            )))
        }
        "traktOAuthErrorCode" => opt_str(services::trakt::trakt_oauth_error_code(&arg_str(
            args_json, "body",
        )?)),
        "traktHistoryRequest" => {
            let args = object(args_json)?;
            opt_json(services::trakt::trakt_history_request_json(
                field_str(&args, "metaJson")?,
                field_str(&args, "episodesJson")?,
            ))
        }
        "traktCollectionBody" => opt_json(services::trakt::trakt_collection_body_json(args_json)),
        // args_json IS the items array for single-array-arg methods
        "traktPlaybackItemsToLibrary" => opt_json(
            services::trakt::trakt_playback_items_to_library_json(args_json),
        ),
        "traktWatchedShowsToItems" => opt_json(services::trakt::trakt_watched_shows_to_items_json(
            args_json,
        )),
        "traktWatchlistToItems" => {
            let args = object(args_json)?;
            opt_json(services::trakt::trakt_watchlist_to_items_json(
                field_str(&args, "moviesJson")?,
                field_str(&args, "showsJson")?,
            ))
        }
        "traktWatchedToIds" => {
            let args = object(args_json)?;
            opt_json(services::trakt::trakt_watched_to_ids_json(
                field_str(&args, "moviesJson")?,
                field_str(&args, "showsJson")?,
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
            let ids_json = catalog::identity::build_trakt_ids_json(field_str(&args, "videoId")?)
                .ok_or_else(|| fail(ErrorKind::NotFound, "could not build trakt ids"))?;
            opt_json(player::playback::scrobble::trakt_scrobble_plan_json(
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
        "traktPlaybackItemsDedup" => {
            opt_json(services::trakt::trakt_playback_items_dedup_json(args_json))
        }
        "traktRemapVideoIds" => opt_json(services::trakt::trakt_remap_video_ids_json(args_json)),
        "traktUpNextToItems" => opt_json(services::trakt::trakt_up_next_to_items_json(args_json)),
        "traktMarkWatchedBody" => {
            opt_json(services::trakt::trakt_mark_watched_body_json(args_json))
        }
        "traktRelatedLookupSlug" => {
            let args = object(args_json)?;
            opt_json(services::trakt::trakt_related_lookup_slug(
                field_str(&args, "lookupJson")?,
                field_str(&args, "wantType")?,
            ))
        }
        "traktRelatedItemsToMetas" => {
            let args = object(args_json)?;
            opt_json(services::trakt::trakt_related_items_to_metas_json(
                field_str(&args, "relatedJson")?,
                field_str(&args, "contentType")?,
            ))
        }
        "traktCommentsRequest" => opt_json(services::trakt::trakt_comments_request_json(args_json)),
        "traktActivityDiff" => opt_json(services::trakt::trakt_activity_diff_json(args_json)),
        "traktCalendarPlan" => opt_json(services::trakt::trakt_calendar_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
