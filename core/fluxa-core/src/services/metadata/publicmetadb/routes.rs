use crate::ffi::*;
use crate::services;

pub(crate) fn route_publicmetadb(method: &str, args_json: &str) -> Outcome {
    match method {
        "publicmetadbEpisodeRatingsBatchCreatePlan" => opt_json(
            services::publicmetadb::publicmetadb_episode_ratings_batch_create_plan(args_json),
        ),
        "publicmetadbEpisodeRatingsBatchDeletePlan" => opt_json(
            services::publicmetadb::publicmetadb_episode_ratings_batch_delete_plan(args_json),
        ),

        "publicmetadbAnimeSeasonsDeleteMappingPlan" => opt_json(
            services::publicmetadb::publicmetadb_anime_seasons_delete_mapping_plan(args_json),
        ),
        "publicmetadbAnimeSeasonsDeleteChunkPlan" => opt_json(
            services::publicmetadb::publicmetadb_anime_seasons_delete_chunk_plan(&arg_str(
                args_json, "id",
            )?),
        ),

        _ => Err(unknown_method()),
    }
}
