use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_content_identity(method: &str, args_json: &str) -> Outcome {
    match method {
        "contentImdbId" => Ok(json!(crate::catalog::identity::imdb_id(&arg_str(
            args_json, "id"
        )?))),
        "streamRequestIds" => {
            let args = object(args_json)?;
            let detail_id = args.get("detailId").and_then(Value::as_str);
            let current_series_lookup_id =
                args.get("currentSeriesLookupId").and_then(Value::as_str);
            let canonical_base_id = args.get("canonicalBaseId").and_then(Value::as_str);
            Ok(json!(crate::catalog::identity::stream_request_ids(
                field_str(&args, "contentType")?,
                field_str(&args, "id")?,
                detail_id,
                current_series_lookup_id,
                canonical_base_id,
            )))
        }
        "cs3PluginFeedKey" => Ok(Value::String(
            crate::catalog::identity::cs3_plugin_feed_key(&arg_str(args_json, "apiName")?),
        )),
        "cs3CatalogFeedKey" => {
            let args = object(args_json)?;
            let catalog_index = field(&args, "catalogIndex")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "catalogIndex must be a number"))?;
            Ok(Value::String(
                crate::catalog::identity::cs3_catalog_feed_key(
                    field_str(&args, "pluginName")?,
                    field_str(&args, "catalogName")?,
                    catalog_index as i32,
                ),
            ))
        }
        "cs3MetadataFeedOptions" => opt_json(
            crate::catalog::identity::cs3_metadata_feed_options_json(args_json),
        ),
        "shortenSynopsis" => Ok(Value::String(crate::catalog::identity::shorten_synopsis(
            &arg_str(args_json, "text")?,
        ))),
        _ => Err(unknown_method()),
    }
}
