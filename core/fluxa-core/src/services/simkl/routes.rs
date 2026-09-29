use crate::ffi::*;
use crate::player;
use crate::services;
use serde_json::{Value, json};

pub(crate) fn route_simkl(method: &str, args_json: &str) -> Outcome {
    match method {
        "simklHistoryRequest" => opt_json(services::simkl::simkl_history_request_json(args_json)),
        "simklWatchlistRequest" => opt_json(services::simkl::simkl_watchlist_request_json(
            args_json, false,
        )),
        "simklWatchlistRemovalRequest" => opt_json(services::simkl::simkl_watchlist_request_json(
            args_json, true,
        )),
        "simklMarkWatchedBody" => {
            opt_json(services::simkl::simkl_mark_watched_body_json(args_json))
        }
        "simklPlaybackDeleteIds" => {
            opt_json(services::simkl::simkl_playback_delete_ids_json(args_json))
        }
        "simklPlaybackItemToContinueMeta" => opt_json(
            services::simkl::simkl_playback_item_to_continue_meta_json(args_json),
        ),
        "simklWatchlistBody" => opt_json(services::simkl::simkl_watchlist_body_json(args_json)),
        "simklWatchingToItems" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_watching_to_items_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklWatchlistToItems" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_watchlist_to_items_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklLibraryToItems" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_library_to_items_json(
                field_str(&args, "showsJson")?,
                field_str(&args, "moviesJson")?,
            ))
        }
        "simklMergePlaybackProgress" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_merge_playback_progress_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "playbackJson")?,
            ))
        }
        "simklWatchedToIds" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_watched_to_ids_json(
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
            opt_json(player::scrobble::simkl_scrobble_body_json(
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
            opt_json(services::simkl::simkl_match_episode_json(
                field_str(&args, "episodesJson")?,
                field_str(&args, "targetJson")?,
            ))
        }
        "simklLookupIdForType" => {
            let args = object(args_json)?;
            match services::simkl::simkl_lookup_id_for_type(
                field_str(&args, "lookupJson")?,
                field_str(&args, "wantType")?,
            ) {
                Some(id) => Ok(json!(id)),
                None => Ok(Value::Null),
            }
        }
        "simklRecommendationCandidates" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_recommendation_candidates_json(
                field_str(&args, "detailJson")?,
            ))
        }
        "simklRecommendationToMeta" => {
            let args = object(args_json)?;
            opt_json(services::simkl::simkl_recommendation_to_meta_json(
                field_str(&args, "recJson")?,
                field_str(&args, "resolvedImdb")?,
            ))
        }
        "simklSyncPlan" => opt_json(services::simkl::simkl_sync_plan_json(args_json)),
        "simklCalendarPlan" => opt_json(services::simkl::simkl_calendar_plan_json(args_json)),
        "simklSyncApply" => opt_json(services::simkl::simkl_sync_apply_json(args_json)),
        _ => Err(unknown_method()),
    }
}
