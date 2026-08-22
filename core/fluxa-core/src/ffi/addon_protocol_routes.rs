use super::*;

pub(super) fn route_addon_protocol(method: &str, args_json: &str) -> Outcome {
    match method {
        "identity" => Ok(Value::String(addon_protocol::identity(&arg_str(
            args_json, "url",
        )?))),
        "normalizeManifestUrl" => Ok(Value::String(addon_protocol::normalize_manifest_url(
            &arg_str(args_json, "url")?,
        ))),
        "manifestFetchPlan" => opt_json(addon_protocol::manifest_fetch_plan_json(&arg_str(
            args_json, "url",
        )?)),
        "baseUrl" => Ok(Value::String(addon_protocol::base_url(&arg_str(
            args_json, "url",
        )?))),
        "preferHttpsAssetUrl" => Ok(json!(addon_protocol::prefer_https_asset_url(&arg_str(
            args_json, "url",
        )?))),
        "manifestCandidates" => Ok(json!(addon_protocol::manifest_candidates(&arg_str(
            args_json, "url",
        )?))),
        "parseManifest" => {
            let args = object(args_json)?;
            opt_json(addon_protocol::parse_manifest(
                field_str(&args, "body")?,
                field_str(&args, "transportUrl")?,
                field_str(&args, "unknownName")?,
            ))
        }
        // args_json IS the descriptor object
        "resolveManifestAssets" => {
            opt_json(addon_protocol::resolve_manifest_assets_json(args_json))
        }
        "mergeLiveManifest" => {
            let args = object(args_json)?;
            let live = args.get("live").and_then(Value::as_str).map(str::to_string);
            let name = args
                .get("unknownName")
                .and_then(Value::as_str)
                .unwrap_or("Unknown Addon");
            opt_json(addon_protocol::merge_live_manifest_json(
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
            Ok(Value::String(addon_protocol::build_resource_url(
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
            Ok(json!(addon_protocol::supports_resource(
                field_str(&args, "manifest")?,
                field_str(&args, "resource")?,
                content_type.as_deref(),
                id.as_deref(),
            )))
        }
        "catalogSupportsExtra" => {
            let args = object(args_json)?;
            Ok(json!(addon_protocol::catalog_supports_extra(
                field_str(&args, "catalog")?,
                field_str(&args, "extraName")?,
            )))
        }
        "normalizeAddonDescriptor" => opt_json(addon_protocol::normalize_addon_descriptor_json(
            &arg_str(args_json, "addonJson")?,
        )),
        "catalogRequiresExtra" => {
            let args = object(args_json)?;
            Ok(json!(addon_protocol::catalog_requires_extra(
                field_str(&args, "catalog")?,
                field_str(&args, "extraName")?,
            )))
        }
        "catalogHasRequiredExtraExcept" => {
            let args = object(args_json)?;
            Ok(json!(addon_protocol::catalog_has_required_extra_except(
                field_str(&args, "catalog")?,
                field_str(&args, "allowedNames")?,
            )))
        }
        // args_json IS the links array
        "classifyMetaLinks" => opt_json(addon_protocol::classify_meta_links_json(args_json)),

        _ => Err(unknown_method()),
    }
}
