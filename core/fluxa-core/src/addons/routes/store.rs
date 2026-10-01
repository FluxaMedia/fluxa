use crate::ffi::*;
use crate::profile;
use serde_json::Value;

pub(crate) fn route_core_contract(method: &str, _args_json: &str) -> Outcome {
    match method {
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
        // args_json IS the profile object
        "profileLocalAddonsKey" => {
            opt_str(crate::addons::sources::store::profile_local_addons_key_json(args_json))
        }
        // args_json IS the request object
        "filterEnabledAddons" => opt_json(
            crate::addons::sources::store::filter_enabled_addons_json(args_json),
        ),
        // args_json IS the { profiles, activeProfileId } request object
        "effectiveAddonsOwnerId" => {
            opt_str(crate::addons::sources::store::effective_addons_owner_id_json(args_json))
        }
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
