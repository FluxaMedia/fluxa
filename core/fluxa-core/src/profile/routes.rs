use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_profile_contract(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        "accountSource" => opt_json(crate::profile::plans::account_source_json(args_json)),
        "activeProfilePlan" => opt_json(crate::profile::plans::active_profile_plan_json(args_json)),
        "tokenMergePlan" => opt_json(crate::profile::plans::token_merge_plan_json(args_json)),
        "profileSyncMergePlan" => opt_json(crate::profile::plans::profile_sync_merge_plan_json(
            args_json,
        )),
        "profileDefaultSeed" => {
            opt_json(crate::profile::plans::profile_default_seed_json(args_json))
        }
        "profileSettingsMigrationPlan" => opt_json(
            crate::profile::plans::profile_settings_migration_plan_json(args_json),
        ),
        "profileAvatarDefault" => opt_json(crate::profile::plans::profile_avatar_default_json(
            args_json,
        )),
        "profileMutationPlan" => {
            opt_json(crate::profile::plans::profile_mutation_plan_json(args_json))
        }
        "createProfilePlan" => opt_json(crate::profile::plans::create_profile_plan_json(args_json)),
        // args_json IS the profiles array
        "primaryProfileId" => opt_str(crate::profile::plans::primary_profile_id_json(args_json)),
        "profilePinHash" => Ok(Value::String(crate::profile::plans::profile_pin_hash(
            &arg_str(args_json, "pin")?,
        ))),
        "profilePinMatches" => {
            let args = object(args_json)?;
            Ok(json!(crate::profile::plans::profile_pin_matches(
                field_str(&args, "profileJson")?,
                field_str(&args, "pin")?
            )))
        }
        "profileConnectionState" => {
            let args = object(args_json)?;
            into_json(crate::profile::plans::profile_connection_state_json(
                field_str(&args, "profileJson")?,
                field(&args, "nowEpochSeconds")?.as_i64().unwrap_or(0),
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_profile_prefs(method: &str, args_json: &str) -> Outcome {
    match method {
        "safePlayerBufferCacheMb" => {
            let args = object(args_json)?;
            let value = args.get("value").and_then(Value::as_i64).map(|v| v as i32);
            Ok(json!(crate::profile::prefs::safe_player_buffer_cache_mb(
                value
            )))
        }
        "safeStreamSourceSelectionMode" => {
            let args = object(args_json)?;
            let mode = args.get("mode").and_then(Value::as_str);
            Ok(Value::String(
                crate::profile::prefs::safe_stream_source_selection_mode(mode).to_string(),
            ))
        }
        // args_json IS the profile object
        "profileSafePrefs" => opt_json(crate::profile::prefs::profile_safe_prefs_json(args_json)),

        _ => Err(unknown_method()),
    }
}
