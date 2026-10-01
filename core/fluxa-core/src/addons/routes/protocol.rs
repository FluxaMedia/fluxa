use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_addon_protocol(method: &str, args_json: &str) -> Outcome {
    match method {
        "identity" => Ok(Value::String(crate::addons::sources::protocol::identity(
            &arg_str(args_json, "url")?,
        ))),
        "manifestFetchPlan" => opt_json(
            crate::addons::sources::protocol::manifest_fetch_plan_json(&arg_str(args_json, "url")?),
        ),
        "baseUrl" => Ok(Value::String(crate::addons::sources::protocol::base_url(
            &arg_str(args_json, "url")?,
        ))),
        "parseManifest" => {
            let args = object(args_json)?;
            opt_json(crate::addons::sources::protocol::parse_manifest(
                field_str(&args, "body")?,
                field_str(&args, "transportUrl")?,
                field_str(&args, "unknownName")?,
            ))
        }
        // args_json IS the descriptor object
        "resolveManifestAssets" => {
            opt_json(crate::addons::sources::protocol::resolve_manifest_assets_json(args_json))
        }
        "buildResourceUrl" => {
            let args = object(args_json)?;
            let extra = args
                .get("extraJson")
                .and_then(Value::as_str)
                .map(str::to_string);
            Ok(Value::String(
                crate::addons::sources::protocol::build_resource_url(
                    field_str(&args, "transportUrl")?,
                    field_str(&args, "resource")?,
                    field_str(&args, "contentType")?,
                    field_str(&args, "id")?,
                    extra.as_deref(),
                ),
            ))
        }
        "normalizeAddonDescriptor" => opt_json(
            crate::addons::sources::protocol::normalize_addon_descriptor_json(&arg_str(
                args_json,
                "addonJson",
            )?),
        ),
        // args_json IS the links array
        _ => Err(unknown_method()),
    }
}
