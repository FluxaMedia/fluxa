use super::*;

pub(super) fn route_addon_resource(method: &str, args_json: &str) -> Outcome {
    match method {
        "parseAddonResourceResult" => {
            let args = object(args_json)?;
            let body = args.get("body").and_then(Value::as_str).map(str::to_string);
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?
                as i32;
            into_json(addon_resource::parse_addon_resource_result_json(
                field_str(&args, "resource")?,
                field_str(&args, "url")?,
                status_code,
                body.as_deref(),
            ))
        }
        "parseAddonStreamResult" => {
            let args = object(args_json)?;
            let body = args.get("body").and_then(Value::as_str).map(str::to_string);
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?
                as i32;
            into_json(addon_resource::parse_addon_stream_result_json(
                field_str(&args, "url")?,
                status_code,
                body.as_deref(),
                field_str(&args, "addonName")?,
            ))
        }
        "normalizeAddonSubtitles" => {
            let args = object(args_json)?;
            into_json(addon_resource::normalize_addon_subtitles_json(
                field_str(&args, "subtitles")?,
                field_str(&args, "resourceUrl")?,
            ))
        }
        "parseCatalogItems" => opt_json(addon_resource::parse_catalog_items_json(
            &arg_str(args_json, "body")?,
            &arg_str(args_json, "fallbackType")?,
        )),
        "parseDirectStreams" => opt_json(addon_resource::parse_direct_streams_json(&arg_str(
            args_json, "body",
        )?)),
        _ => Err(unknown_method()),
    }
}
