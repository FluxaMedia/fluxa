use super::*;

pub(super) fn route_profile_contract(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        "activeProfilePlan" => opt_json(profile_contract::active_profile_plan_json(args_json)),
        "tokenMergePlan" => opt_json(profile_contract::token_merge_plan_json(args_json)),
        "profileDefaultSeed" => opt_json(profile_contract::profile_default_seed_json(args_json)),
        "profileSettingsMigrationPlan" => opt_json(
            profile_contract::profile_settings_migration_plan_json(args_json),
        ),
        "profileAvatarDefault" => {
            opt_json(profile_contract::profile_avatar_default_json(args_json))
        }
        "profileMutationPlan" => opt_json(profile_contract::profile_mutation_plan_json(args_json)),
        "createProfilePlan" => opt_json(profile_contract::create_profile_plan_json(args_json)),
        // args_json IS the profiles array
        "primaryProfileId" => opt_str(profile_contract::primary_profile_id_json(args_json)),
        "profilePinHash" => Ok(Value::String(profile_contract::profile_pin_hash(&arg_str(
            args_json, "pin",
        )?))),
        "profilePinMatches" => {
            let args = object(args_json)?;
            Ok(json!(profile_contract::profile_pin_matches(
                field_str(&args, "profileJson")?,
                field_str(&args, "pin")?
            )))
        }
        "profileConnectionState" => {
            let args = object(args_json)?;
            into_json(profile_contract::profile_connection_state_json(
                field_str(&args, "profileJson")?,
                field(&args, "nowEpochSeconds")?.as_i64().unwrap_or(0),
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_profile_prefs(method: &str, args_json: &str) -> Outcome {
    match method {
        "safePlayerBufferCacheMb" => {
            let args = object(args_json)?;
            let value = args.get("value").and_then(Value::as_i64).map(|v| v as i32);
            Ok(json!(profile_prefs::safe_player_buffer_cache_mb(value)))
        }
        "safeDolbyVisionFallbackMode" => {
            let args = object(args_json)?;
            let mode = args.get("mode").and_then(Value::as_str);
            let legacy_dv7_fallback = args.get("legacyDv7Fallback").and_then(Value::as_bool);
            let legacy_dv7_to_dv8_fallback =
                args.get("legacyDv7ToDv8Fallback").and_then(Value::as_bool);
            Ok(Value::String(
                profile_prefs::safe_dolby_vision_fallback_mode(
                    mode,
                    legacy_dv7_fallback,
                    legacy_dv7_to_dv8_fallback,
                )
                .to_string(),
            ))
        }
        "safeStreamSourceSelectionMode" => {
            let args = object(args_json)?;
            let mode = args.get("mode").and_then(Value::as_str);
            Ok(Value::String(
                profile_prefs::safe_stream_source_selection_mode(mode).to_string(),
            ))
        }
        // args_json IS the profile object
        "profileSafePrefs" => opt_json(profile_prefs::profile_safe_prefs_json(args_json)),

        _ => Err(unknown_method()),
    }
}
