use crate::ffi::*;
use crate::services;

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
        _ => Err(unknown_method()),
    }
}
