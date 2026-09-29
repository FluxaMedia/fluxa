use crate::ffi::*;
use crate::services;
use serde_json::{Value, json};

pub(crate) fn route_anilist(method: &str, args_json: &str) -> Outcome {
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
            Ok(services::anilist::anilist_entries_to_sync(
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
            Ok(services::anilist::merge_library_items_by_id(
                local, incoming,
            ))
        }
        "anilistGraphqlQueries" => Ok(Value::String(
            services::anilist::anilist_graphql_queries_json(),
        )),
        "anilistSaveMediaListEntryVariables" => {
            let args = object(args_json)?;
            let progress = args.get("progress").and_then(Value::as_i64);
            opt_json(
                services::anilist::anilist_save_media_list_entry_variables_json(
                    field_str(&args, "contentId")?,
                    field_str(&args, "status")?,
                    progress,
                ),
            )
        }
        // args_json IS the meta object
        "extractAnilistIdFromLinks" => Ok(json!(services::anilist::extract_anilist_id_from_links(
            &object(args_json)?
        ))),
        "anilistSearchBestMatch" => {
            opt_json(services::anilist::anilist_search_best_match_json(args_json))
        }
        "anilistMediaListStatus" => {
            let args = object(args_json)?;
            let total_episodes = field(&args, "totalEpisodes")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "totalEpisodes must be a number"))?;
            let progress_episode = field(&args, "progressEpisode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "progressEpisode must be a number"))?;
            Ok(json!(services::anilist::anilist_media_list_status(
                total_episodes,
                progress_episode
            )))
        }
        "anilistCalendarPlan" => opt_json(services::anilist::anilist_calendar_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
