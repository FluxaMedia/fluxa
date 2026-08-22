use super::*;

pub(super) fn route_publicmetadb(method: &str, args_json: &str) -> Outcome {
    match method {
        "publicmetadbBearer" => Ok(Value::String(publicmetadb_plan::publicmetadb_bearer(
            &arg_str(args_json, "apiKey")?,
        ))),

        "publicmetadbResumeUrl" => Ok(Value::String(publicmetadb_plan::publicmetadb_resume_url(
            args_json,
        ))),
        "publicmetadbResumeSavePlan" => {
            opt_json(publicmetadb_plan::publicmetadb_resume_save_plan(args_json))
        }
        "publicmetadbResumeBatchPlan" => {
            opt_json(publicmetadb_plan::publicmetadb_resume_batch_plan(args_json))
        }
        "publicmetadbResumeDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_resume_delete_plan(&arg_str(args_json, "id")?),
        ),

        "publicmetadbWatchedUrl" => Ok(Value::String(publicmetadb_plan::publicmetadb_watched_url(
            args_json,
        ))),
        "publicmetadbWatchedMarkPlan" => {
            let args = object(args_json)?;
            let dedupe = args.get("dedupe").and_then(Value::as_bool).unwrap_or(false);
            opt_json(publicmetadb_plan::publicmetadb_watched_mark_plan(
                args_json, dedupe,
            ))
        }
        "publicmetadbWatchedEditDatePlan" => {
            let args = object(args_json)?;
            opt_json(publicmetadb_plan::publicmetadb_watched_edit_date_plan(
                field_str(&args, "id")?,
                args_json,
            ))
        }
        "publicmetadbWatchedDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_watched_delete_plan(&arg_str(args_json, "id")?),
        ),
        "publicmetadbWatchedBulkDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_watched_bulk_delete_plan(args_json),
        ),

        "publicmetadbSkipsUrl" => opt_str(publicmetadb_plan::publicmetadb_skips_url(args_json)),
        "publicmetadbSkipsCreatePlan" => {
            opt_json(publicmetadb_plan::publicmetadb_skips_create_plan(args_json))
        }
        "publicmetadbSkipsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_skips_delete_plan(&arg_str(args_json, "id")?),
        ),

        "publicmetadbRatingsUrl" => opt_str(publicmetadb_plan::publicmetadb_ratings_url(args_json)),
        "publicmetadbRatingsCreatePlan" => opt_json(
            publicmetadb_plan::publicmetadb_ratings_create_plan(args_json),
        ),
        "publicmetadbRatingsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_ratings_delete_plan(&arg_str(args_json, "id")?),
        ),

        "publicmetadbEpisodeRatingsUrl" => opt_str(
            publicmetadb_plan::publicmetadb_episode_ratings_url(args_json),
        ),
        "publicmetadbEpisodeRatingsCreatePlan" => opt_json(
            publicmetadb_plan::publicmetadb_episode_ratings_create_plan(args_json),
        ),
        "publicmetadbEpisodeRatingsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_episode_ratings_delete_plan(&arg_str(args_json, "id")?),
        ),
        "publicmetadbEpisodeRatingsBatchUrl" => opt_str(
            publicmetadb_plan::publicmetadb_episode_ratings_batch_url(args_json),
        ),
        "publicmetadbEpisodeRatingsBatchCreatePlan" => {
            opt_json(publicmetadb_plan::publicmetadb_episode_ratings_batch_create_plan(args_json))
        }
        "publicmetadbEpisodeRatingsBatchDeletePlan" => {
            opt_json(publicmetadb_plan::publicmetadb_episode_ratings_batch_delete_plan(args_json))
        }

        "publicmetadbHighlightsUrl" => {
            opt_str(publicmetadb_plan::publicmetadb_highlights_url(args_json))
        }
        "publicmetadbHighlightsCreatePlan" => opt_json(
            publicmetadb_plan::publicmetadb_highlights_create_plan(args_json),
        ),
        "publicmetadbHighlightsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_highlights_delete_plan(&arg_str(args_json, "id")?),
        ),

        "publicmetadbMappingsUrl" => {
            opt_str(publicmetadb_plan::publicmetadb_mappings_url(args_json))
        }
        "publicmetadbMappingsLookupUrl" => opt_str(
            publicmetadb_plan::publicmetadb_mappings_lookup_url(args_json),
        ),
        "publicmetadbMappingsCreatePlan" => opt_json(
            publicmetadb_plan::publicmetadb_mappings_create_plan(args_json),
        ),
        "publicmetadbMappingsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_mappings_delete_plan(&arg_str(args_json, "id")?),
        ),

        "publicmetadbAnimeSeasonsUrl" => {
            opt_str(publicmetadb_plan::publicmetadb_anime_seasons_url(args_json))
        }
        "publicmetadbAnimeSeasonsSubmitPlan" => opt_json(
            publicmetadb_plan::publicmetadb_anime_seasons_submit_plan(args_json),
        ),
        "publicmetadbAnimeSeasonsDeleteMappingPlan" => {
            opt_json(publicmetadb_plan::publicmetadb_anime_seasons_delete_mapping_plan(args_json))
        }
        "publicmetadbAnimeSeasonsDeleteChunkPlan" => opt_json(
            publicmetadb_plan::publicmetadb_anime_seasons_delete_chunk_plan(&arg_str(
                args_json, "id",
            )?),
        ),

        "publicmetadbListsUrl" => Ok(Value::String(publicmetadb_plan::publicmetadb_lists_url(
            args_json,
        ))),
        "publicmetadbListsCreatePlan" => {
            opt_json(publicmetadb_plan::publicmetadb_lists_create_plan(args_json))
        }
        "publicmetadbListsDeletePlan" => opt_json(
            publicmetadb_plan::publicmetadb_lists_delete_plan(&arg_str(args_json, "listId")?),
        ),
        "publicmetadbListItemsUrl" => {
            let args = object(args_json)?;
            opt_str(publicmetadb_plan::publicmetadb_list_items_url(
                field_str(&args, "listId")?,
                args_json,
            ))
        }
        "publicmetadbListItemsAddPlan" => {
            let args = object(args_json)?;
            opt_json(publicmetadb_plan::publicmetadb_list_items_add_plan(
                field_str(&args, "listId")?,
                args_json,
            ))
        }
        "publicmetadbListItemsRemovePlan" => {
            let args = object(args_json)?;
            opt_json(publicmetadb_plan::publicmetadb_list_items_remove_plan(
                field_str(&args, "listId")?,
                field_str(&args, "itemId")?,
            ))
        }

        "publicmetadbCatalogsUrl" => {
            Ok(Value::String(publicmetadb_plan::publicmetadb_catalogs_url()))
        }
        "publicmetadbCatalogItemsUrl" => {
            let args = object(args_json)?;
            opt_str(publicmetadb_plan::publicmetadb_catalog_items_url(
                field_str(&args, "id")?,
                args_json,
            ))
        }

        "publicmetadbVotesUrl" => {
            let args = object(args_json)?;
            let all = args.get("all").and_then(Value::as_bool).unwrap_or(false);
            opt_str(publicmetadb_plan::publicmetadb_votes_url(
                field_str(&args, "resource")?,
                field_str(&args, "itemId")?,
                all,
            ))
        }
        "publicmetadbVotesCreatePlan" => {
            let args = object(args_json)?;
            opt_json(publicmetadb_plan::publicmetadb_votes_create_plan(
                field_str(&args, "resource")?,
                field_str(&args, "itemId")?,
                field_i64(&args, "vote")?,
            ))
        }
        "publicmetadbVotesDeletePlan" => {
            let args = object(args_json)?;
            opt_json(publicmetadb_plan::publicmetadb_votes_delete_plan(
                field_str(&args, "resource")?,
                field_str(&args, "itemId")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
