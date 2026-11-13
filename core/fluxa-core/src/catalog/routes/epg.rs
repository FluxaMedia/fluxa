use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_epg(method: &str, args_json: &str) -> Outcome {
    let args = object(args_json)?;
    let int = |key: &str| args.get(key).and_then(Value::as_i64).unwrap_or_default();
    match method {
        "epgGuideCatalogs" => Ok(Value::Array(crate::catalog::epg::guide_catalogs(&args))),
        "epgUtcDays" => Ok(json!(crate::catalog::epg::utc_days(
            int("fromMs"),
            int("toMs")
        ))),
        "epgBuildGuide" => {
            let pages = args
                .get("pages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            Ok(crate::catalog::epg::build_guide(
                &pages,
                int("fromMs"),
                int("toMs"),
                int("nowMs"),
            ))
        }
        "epgRefreshDelayMs" => Ok(json!(crate::catalog::epg::refresh_delay_ms(
            args.get("lastEndMs").and_then(Value::as_i64),
            int("nowMs"),
            args.get("failed").and_then(Value::as_bool) == Some(true),
        ))),
        _ => Err(unknown_method()),
    }
}
