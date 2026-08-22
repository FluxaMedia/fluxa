use super::*;

pub(super) fn route_tmdb(method: &str, args_json: &str) -> Outcome {
    match method {
        "tmdbContentType" => Ok(Value::String(
            tmdb_plan::tmdb_content_type(&arg_str(args_json, "contentType")?).to_string(),
        )),
        "tmdbLanguage" => Ok(Value::String(tmdb_plan::tmdb_language(&arg_str(
            args_json, "language",
        )?))),
        "tmdbImageUrl" => {
            let args = object(args_json)?;
            Ok(json!(tmdb_plan::tmdb_image_url(
                args.get("path").and_then(Value::as_str),
                field_str(&args, "size")?,
            )))
        }
        "tmdbMetaToMeta" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::tmdb_meta_to_meta_json(
                field_str(&args, "itemJson")?,
                field_str(&args, "requestedType")?,
                field_str(&args, "language")?,
            ))
        }
        // args_json IS the video/items JSON for single-arg methods
        "tmdbVideoToTrailer" => opt_json(tmdb_plan::tmdb_video_to_trailer_json(args_json)),
        "tmdbBulkMetas" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::tmdb_bulk_metas_to_metas_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "requestedType")?,
                field_str(&args, "language")?,
            ))
        }
        "tmdbBulkVideosToTrailers" => {
            opt_json(tmdb_plan::tmdb_bulk_videos_to_trailers_json(args_json))
        }
        "tmdbResolveIdHint" => {
            let (content_type, is_movie) =
                tmdb_plan::tmdb_resolve_id_hint(&arg_str(args_json, "contentId")?);
            Ok(json!([content_type, is_movie]))
        }
        "tmdbPeopleRequestPlan" => {
            let args = object(args_json)?;
            Ok(tmdb_plan::tmdb_people_request_plan(
                field(&args, "meta")?,
                field_str(&args, "apiKey")?,
                field_str(&args, "language")?,
            ))
        }
        "tmdbCreditsUrlFromFind" => {
            let args = object(args_json)?;
            Ok(json!(tmdb_plan::tmdb_credits_url_from_find(
                field(&args, "find")?,
                field(&args, "meta")?,
                field_str(&args, "apiKey")?,
                field_str(&args, "language")?,
            )))
        }
        "tmdbBuiltinManifest" => Ok(Value::String(tmdb_plan::tmdb_builtin_manifest_json())),
        "tmdbBuiltinCatalogUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(tmdb_plan::tmdb_builtin_catalog_url(
                field_str(&args, "contentType")?,
                field(&args, "extra")?,
                field_str(&args, "apiKey")?,
                field_str(&args, "language")?,
            )))
        }
        "tmdbBuiltinMetaRequestPlan" => {
            opt_json(tmdb_plan::tmdb_builtin_meta_request_plan_json(args_json))
        }
        "tmdbBuiltinMetaUrls" => opt_json(tmdb_plan::tmdb_builtin_meta_urls_json(args_json)),
        "tmdbSeasonRequestUrl" => {
            let args = object(args_json)?;
            let season = field(&args, "season")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "season must be a number"))?;
            Ok(json!(tmdb_plan::tmdb_season_request_url(
                field_str(&args, "contentId")?,
                season,
                field_str(&args, "apiKey")?,
                field_str(&args, "language")?,
            )))
        }
        "tmdbBuiltinMetaUrlsFromFind" => {
            opt_json(tmdb_plan::tmdb_builtin_meta_urls_from_find_json(args_json))
        }
        "tmdbDetailRequestPlan" => opt_json(tmdb_plan::tmdb_detail_request_plan_json(args_json)),
        "tmdbDetailRequestUrlsFromFind" => opt_json(
            tmdb_plan::tmdb_detail_request_urls_from_find_json(args_json),
        ),
        "tmdbFullMetaToMeta" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::tmdb_full_meta_to_meta_json(
                field_str(&args, "detailsJson")?,
                field_str(&args, "creditsJson")?,
                field_str(&args, "imagesJson")?,
                field_str(&args, "externalIdsJson")?,
                field_str(&args, "extrasJson")?,
                field_str(&args, "requestedType")?,
                field_str(&args, "language")?,
            ))
        }
        "tmdbPickLogo" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::tmdb_pick_logo_json(
                field_str(&args, "imagesJson")?,
                field_str(&args, "language")?,
            ))
        }
        "tmdbEpisodesToVideos" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::tmdb_episodes_to_videos_json(
                field_str(&args, "seasonJson")?,
                field_str(&args, "seriesId")?,
            ))
        }
        "tmdbMergeEnrichment" => {
            let args = object(args_json)?;
            opt_json(tmdb_plan::merge_tmdb_enrichment_json(
                field_str(&args, "baseJson")?,
                field_str(&args, "tmdbJson")?,
                field_str(&args, "flagsJson")?,
            ))
        }
        "tmdbPeopleImagesFromCredits" => {
            let args = object(args_json)?;
            let empty: Vec<Value> = Vec::new();
            let links = args
                .get("links")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            Ok(tmdb_plan::tmdb_people_images_from_credits(
                field(&args, "credits")?,
                links,
            ))
        }

        _ => Err(unknown_method()),
    }
}
