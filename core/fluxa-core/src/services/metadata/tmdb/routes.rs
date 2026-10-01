use crate::ffi::*;
use crate::services;
use serde_json::Value;

pub(crate) fn route_tmdb(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the video/items JSON for single-arg methods
        "tmdbBulkMetas" => {
            let args = object(args_json)?;
            opt_json(services::tmdb::tmdb_bulk_metas_to_metas_json(
                field_str(&args, "itemsJson")?,
                field_str(&args, "requestedType")?,
                field_str(&args, "language")?,
            ))
        }
        "tmdbBulkVideosToTrailers" => {
            opt_json(services::tmdb::tmdb_bulk_videos_to_trailers_json(args_json))
        }
        "tmdbBuiltinManifest" => Ok(Value::String(services::tmdb::tmdb_builtin_manifest_json())),
        "tmdbBuiltinCatalogUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(services::tmdb::tmdb_builtin_catalog_url(
                field_str(&args, "contentType")?,
                field(&args, "extra")?,
                field_str(&args, "apiKey")?,
                field_str(&args, "language")?,
            )))
        }
        "tmdbDetailRequestPlan" => {
            opt_json(services::tmdb::tmdb_detail_request_plan_json(args_json))
        }
        "tmdbDetailRequestUrlsFromFind" => opt_json(
            services::tmdb::tmdb_detail_request_urls_from_find_json(args_json),
        ),
        _ => Err(unknown_method()),
    }
}
