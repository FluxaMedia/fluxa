use crate::ffi::*;
use crate::services;

pub(crate) fn route_stremio(method: &str, args_json: &str) -> Outcome {
    match method {
        "stremioLibraryMutationPlan" => opt_json(
            services::stremio::stremio_library_mutation_plan_json(args_json),
        ),
        "stremioWatchlistToItems" => opt_json(services::stremio::stremio_watchlist_to_items_json(
            args_json,
        )),
        "stremioWatchedToIds" => {
            opt_json(services::stremio::stremio_watched_to_ids_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
