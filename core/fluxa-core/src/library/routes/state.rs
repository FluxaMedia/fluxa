use crate::ffi::*;
use crate::home;

pub(crate) fn route_library_state(method: &str, args_json: &str) -> Outcome {
    match method {
        "terminalRecommendationPlan" => opt_json(
            home::recommendation::terminal_recommendation_plan_json(args_json),
        ),
        "terminalRecommendationEligibility" => {
            opt_json(home::recommendation::terminal_recommendation_eligibility_json(args_json))
        }
        "recommendationOutroPlan" => opt_json(
            home::recommendation::recommendation_outro_plan_json(args_json),
        ),
        "continueWatchingEpisodeStatus" => opt_json(
            crate::library::continue_watching::continue_watching_episode_status_json(args_json),
        ),
        "continueWatchingSourcePlan" => opt_json(
            crate::library::state::continue_watching_source_plan_json(args_json),
        ),
        "homeMetadataFeedPlan" => opt_json(home::ranking::home_metadata_feed_plan_json(args_json)),
        "annotateCatalogItems" => opt_json(home::ranking::annotate_catalog_items_json(args_json)),
        // args_json IS the items/item/doc JSON for single-arg methods
        "normalizeLibraryDocument" => into_json(
            crate::library::state::normalize_library_document_json(args_json),
        ),
        "buildContinueWatchingFromProgress" => {
            opt_json(crate::library::state::build_continue_watching_from_progress_json(args_json))
        }
        "resolveNextEpisode" => {
            let args = object(args_json)?;
            opt_json(crate::library::state::resolve_next_episode_json(
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
        "buildHomeCollectionShelves" => {
            let args = object(args_json)?;
            opt_json(home::ranking::build_home_collection_shelves_json(
                field_str(&args, "profileJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }
        "homeHeroPlan" => opt_json(home::ranking::home_hero_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
