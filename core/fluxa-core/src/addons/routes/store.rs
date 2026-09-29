use crate::ffi::*;
use crate::{headless_engine, profile};
use serde_json::{Value, json};

pub(crate) fn route_core_contract(method: &str, args_json: &str) -> Outcome {
    match method {
        "coreCapabilities" => into_json(headless_engine::manifest::core_capabilities_json(
            object(args_json)
                .ok()
                .and_then(|o| o.get("portable").and_then(Value::as_bool))
                .unwrap_or(false),
        )),
        "coreContractManifest" => {
            into_json(headless_engine::manifest::core_contract_manifest_json())
        }

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_addon_store(method: &str, args_json: &str) -> Outcome {
    match method {
        "sha256VerificationStatus" => {
            let args = object(args_json)?;
            Ok(Value::String(
                crate::settings::checksum::sha256_verification_status(
                    args.get("expected").and_then(Value::as_str),
                    field_str(&args, "actual")?,
                )
                .to_string(),
            ))
        }
        "addonStoreInputType" => Ok(Value::String(
            crate::addons::sources::store::addon_store_input_type(&arg_str(args_json, "input")?)
                .to_string(),
        )),
        "normalizeCloudstreamRepoUrl" => Ok(Value::String(
            crate::addons::sources::store::normalize_cloudstream_repo_url(&arg_str(
                args_json, "url",
            )?),
        )),
        "normalizeCloudstreamRepoInput" => Ok(Value::String(
            crate::addons::sources::store::normalize_cloudstream_repo_input(&arg_str(
                args_json, "url",
            )?),
        )),
        "normalizePluginRepositoryUrl" => Ok(Value::String(
            crate::addons::sources::store::normalize_plugin_repository_url(&arg_str(
                args_json, "url",
            )?),
        )),
        "isSecureRemoteUrl" => Ok(json!(crate::addons::sources::store::is_secure_remote_url(
            &arg_str(args_json, "url",)?
        ))),
        "samePluginRepositoryUrl" => {
            let args = object(args_json)?;
            Ok(json!(
                crate::addons::sources::store::same_plugin_repository_url(
                    field_str(&args, "left")?,
                    field_str(&args, "right")?,
                )
            ))
        }
        // args_json IS the profile object
        "profileLocalAddonsKey" => {
            opt_str(crate::addons::sources::store::profile_local_addons_key_json(args_json))
        }
        "addonProfileMutationPlan" => {
            opt_json(crate::addons::sources::store::addon_profile_mutation_plan_json(args_json))
        }
        "sanitizeProfile" => {
            let args = object(args_json)?;
            let merge_mirrored_addons = field(&args, "mergeMirroredAddons")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "mergeMirroredAddons must be bool"))?;
            opt_json(crate::addons::sources::store::sanitize_profile_json(
                field_str(&args, "profile")?,
                field_str(&args, "mirroredAddons")?,
                merge_mirrored_addons,
            ))
        }
        // args_json IS the request object
        "addonStoreSearchPolicy" => {
            opt_json(crate::addons::sources::store::addon_store_search_policy_json(args_json))
        }
        "extractAddonManifestUrl" => opt_json(
            crate::addons::sources::store::extract_addon_manifest_url(&arg_str(args_json, "text")?),
        ),
        "filterEnabledAddons" => opt_json(
            crate::addons::sources::store::filter_enabled_addons_json(args_json),
        ),
        "addonStoreEntriesPlan" => {
            opt_json(crate::addons::sources::store::addon_store_entries_plan_json(args_json))
        }
        // args_json IS the { profiles, activeProfileId } request object
        "effectiveAddonsOwnerId" => {
            opt_str(crate::addons::sources::store::effective_addons_owner_id_json(args_json))
        }
        "effectivePluginsOwnerId" => {
            opt_str(crate::addons::sources::store::effective_plugins_owner_id_json(args_json))
        }
        "pluginStorageFallback" => opt_json(
            crate::addons::sources::store::plugin_storage_fallback_json(args_json),
        ),

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_profile_avatar_pack(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these. The platform owns
        // the HTTP calls between plans; this crate only validates and maps the
        // GitHub responses into the stable UI contract.
        "profileAvatarPackManifestPlan" => {
            opt_json(profile::avatar_pack::profile_avatar_pack_manifest_plan_json(args_json))
        }
        "profileAvatarPackRepositoryPlan" => {
            opt_json(profile::avatar_pack::profile_avatar_pack_repository_plan_json(args_json))
        }
        "profileAvatarPackDiscoveryPlan" => {
            opt_json(profile::avatar_pack::profile_avatar_pack_discovery_plan_json(args_json))
        }
        "profileAvatarPackCatalog" => opt_json(
            profile::avatar_pack::profile_avatar_pack_catalog_json(args_json),
        ),
        "profileAvatarPackParse" => {
            opt_json(profile::avatar_pack::profile_avatar_pack_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
