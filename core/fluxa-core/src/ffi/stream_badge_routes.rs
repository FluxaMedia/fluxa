use super::*;

pub(super) fn route_stream_badges(method: &str, args_json: &str) -> Outcome {
    match method {
        "parseStreamBadgeImport" => {
            let args = object(args_json)?;
            let source_url = field_str(&args, "sourceUrl")?;
            let payload = field_str(&args, "payload")?;
            into_json(
                stream_badges::parse_stream_badge_import_json(source_url, payload)
                    .map_err(|message| fail(ErrorKind::InvalidArgs, message))?,
            )
        }
        "normalizeStreamBadgeRules" => into_json(stream_badges::normalize_stream_badge_rules_json(
            &arg_str(args_json, "rulesJson")?,
        )),
        "upsertStreamBadgeImport" => {
            let args = object(args_json)?;
            let rules_json = field_str(&args, "rulesJson")?;
            let import_json = field_str(&args, "importJson")?;
            let activate = args
                .get("activate")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            opt_json(stream_badges::upsert_stream_badge_import_json(
                rules_json,
                import_json,
                activate,
            ))
        }
        "setActiveStreamBadgeSource" => {
            let args = object(args_json)?;
            into_json(stream_badges::set_active_stream_badge_source_json(
                field_str(&args, "rulesJson")?,
                field_str(&args, "sourceUrl")?,
            ))
        }
        "removeStreamBadgeSource" => {
            let args = object(args_json)?;
            into_json(stream_badges::remove_stream_badge_source_json(
                field_str(&args, "rulesJson")?,
                field_str(&args, "sourceUrl")?,
            ))
        }
        "matchStreamBadges" => {
            let args = object(args_json)?;
            into_json(stream_badges::match_stream_badges_json(
                field_str(&args, "streamJson")?,
                field_str(&args, "rulesJson")?,
            ))
        }
        _ => Err(unknown_method()),
    }
}
