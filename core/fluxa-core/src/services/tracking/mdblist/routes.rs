use crate::ffi::*;
use crate::services;
use serde_json::json;

pub(crate) fn route_mdblist(method: &str, args_json: &str) -> Outcome {
    match method {
        "mdblistMediaInfoBatchPlan" => {
            let args = object(args_json)?;
            let ids: Vec<String> = field(&args, "ids")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "ids must be an array"))?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            opt_json(services::mdblist::mdblist_media_info_batch_plan(
                field_str(&args, "provider")?,
                field_str(&args, "mediaType")?,
                &ids,
            ))
        }
        "mdblistMediaRatingsPlan" => {
            let args = object(args_json)?;
            let media_type = services::mdblist::mdblist_content_type(field_str(&args, "contentType")?);
            let url = services::mdblist::mdblist_media_info_url(
                "imdb",
                media_type,
                field_str(&args, "id")?,
                None,
            );
            opt_json(Some(json!({"method": "GET", "url": url}).to_string()))
        }
        "mdblistMediaRatings" => {
            let args = object(args_json)?;
            opt_json(services::mdblist::mdblist_media_ratings_from_response_json(
                &field(&args, "response")?.to_string(),
            ))
        }
        "mdblistUserListsPlan" => {
            let args = object(args_json)?;
            opt_json(Some(
                services::mdblist::mdblist_user_lists_plan(&json!(args)).to_string(),
            ))
        }
        "mdblistUserLists" => {
            let args = object(args_json)?;
            let response = field(&args, "response")?;
            let lists = response
                .as_array()
                .or_else(|| response.get("lists").and_then(|lists| lists.as_array()))
                .into_iter()
                .flatten()
                .filter_map(|list| {
                    Some(json!({
                        "id": list.get("id")?.as_i64()?.to_string(),
                        "name": list.get("name")?.as_str()?,
                    }))
                })
                .collect::<Vec<_>>();
            opt_json(Some(serde_json::Value::Array(lists).to_string()))
        }
        _ => Err(unknown_method()),
    }
}
