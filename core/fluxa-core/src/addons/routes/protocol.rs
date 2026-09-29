use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_addon_protocol(method: &str, args_json: &str) -> Outcome {
    match method {
        "identity" => Ok(Value::String(crate::addons::protocol::identity(&arg_str(
            args_json, "url",
        )?))),
        "normalizeManifestUrl" => Ok(Value::String(
            crate::addons::protocol::normalize_manifest_url(&arg_str(args_json, "url")?),
        )),
        "manifestFetchPlan" => opt_json(crate::addons::protocol::manifest_fetch_plan_json(
            &arg_str(args_json, "url")?,
        )),
        "baseUrl" => Ok(Value::String(crate::addons::protocol::base_url(&arg_str(
            args_json, "url",
        )?))),
        "preferHttpsAssetUrl" => Ok(json!(crate::addons::protocol::prefer_https_asset_url(
            &arg_str(args_json, "url",)?
        ))),
        "manifestCandidates" => Ok(json!(crate::addons::protocol::manifest_candidates(
            &arg_str(args_json, "url",)?
        ))),
        "parseManifest" => {
            let args = object(args_json)?;
            opt_json(crate::addons::protocol::parse_manifest(
                field_str(&args, "body")?,
                field_str(&args, "transportUrl")?,
                field_str(&args, "unknownName")?,
            ))
        }
        // args_json IS the descriptor object
        "resolveManifestAssets" => opt_json(crate::addons::protocol::resolve_manifest_assets_json(
            args_json,
        )),
        "mergeLiveManifest" => {
            let args = object(args_json)?;
            let live = args.get("live").and_then(Value::as_str).map(str::to_string);
            let name = args
                .get("unknownName")
                .and_then(Value::as_str)
                .unwrap_or("Unknown Addon");
            opt_json(crate::addons::protocol::merge_live_manifest_json(
                field_str(&args, "descriptor")?,
                live.as_deref(),
                name,
            ))
        }
        "buildResourceUrl" => {
            let args = object(args_json)?;
            let extra = args
                .get("extraJson")
                .and_then(Value::as_str)
                .map(str::to_string);
            Ok(Value::String(crate::addons::protocol::build_resource_url(
                field_str(&args, "transportUrl")?,
                field_str(&args, "resource")?,
                field_str(&args, "contentType")?,
                field_str(&args, "id")?,
                extra.as_deref(),
            )))
        }
        "supportsResource" => {
            let args = object(args_json)?;
            let content_type = args
                .get("contentType")
                .and_then(Value::as_str)
                .map(str::to_string);
            let id = args.get("id").and_then(Value::as_str).map(str::to_string);
            Ok(json!(crate::addons::protocol::supports_resource(
                field_str(&args, "manifest")?,
                field_str(&args, "resource")?,
                content_type.as_deref(),
                id.as_deref(),
            )))
        }
        "catalogSupportsExtra" => {
            let args = object(args_json)?;
            Ok(json!(crate::addons::protocol::catalog_supports_extra(
                field_str(&args, "catalog")?,
                field_str(&args, "extraName")?,
            )))
        }
        "catalogSearchEligible" => Ok(json!(crate::addons::protocol::catalog_search_eligible(
            field_str(&object(args_json)?, "catalog")?,
        ))),
        "normalizeAddonDescriptor" => {
            opt_json(crate::addons::protocol::normalize_addon_descriptor_json(
                &arg_str(args_json, "addonJson")?,
            ))
        }
        "catalogRequiresExtra" => {
            let args = object(args_json)?;
            Ok(json!(crate::addons::protocol::catalog_requires_extra(
                field_str(&args, "catalog")?,
                field_str(&args, "extraName")?,
            )))
        }
        "catalogHasRequiredExtraExcept" => {
            let args = object(args_json)?;
            Ok(json!(
                crate::addons::protocol::catalog_has_required_extra_except(
                    field_str(&args, "catalog")?,
                    field_str(&args, "allowedNames")?,
                )
            ))
        }
        // args_json IS the links array
        "classifyMetaLinks" => {
            opt_json(crate::addons::protocol::classify_meta_links_json(args_json))
        }

        _ => Err(unknown_method()),
    }
}
