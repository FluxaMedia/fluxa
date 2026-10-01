use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_profile_contract(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        "accountSource" => opt_json(crate::profile::plans::account_source_json(args_json)),
        "activeProfilePlan" => opt_json(crate::profile::plans::active_profile_plan_json(args_json)),
        "tokenMergePlan" => opt_json(crate::profile::plans::token_merge_plan_json(args_json)),
        "profileSettingsMigrationPlan" => opt_json(
            crate::profile::plans::profile_settings_migration_plan_json(args_json),
        ),
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
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_profile_prefs(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the profile object
        _ => Err(unknown_method()),
    }
}
