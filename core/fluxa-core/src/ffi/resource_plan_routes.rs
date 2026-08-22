use super::*;

pub(super) fn route_resource_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // Repository / resource flow — args_json IS the request object
        "addonResourceRequestPlan" => {
            opt_json(repository_flow::addon_resource_request_plan_json(args_json))
        }
        "repositoryMetaDetailPlan" => {
            opt_json(repository_flow::repository_meta_detail_plan_json(args_json))
        }
        "manifestFetchDecision" => {
            opt_json(repository_flow::manifest_fetch_decision_json(args_json))
        }
        "repositorySeasonVideos" => {
            let args = object(args_json)?;
            let season_number = field(&args, "seasonNumber")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "seasonNumber must be a number"))?
                as i32;
            into_json(repository_flow::repository_season_videos_json(
                field_str(&args, "metaDetailJson")?,
                season_number,
            ))
        }
        "addonStreamsWithProvider" => {
            let args = object(args_json)?;
            into_json(repository_flow::addon_streams_with_provider_json(
                field_str(&args, "streamsJson")?,
                field_str(&args, "addonName")?,
            ))
        }
        "resourceFetchPlan" => opt_json(platform_plan::resource_fetch_plan_json(args_json)),
        "resourceFetchExecutionPolicy" => opt_json(
            platform_plan::resource_fetch_execution_policy_json(args_json),
        ),
        "resourceParsePlan" => opt_json(platform_plan::resource_parse_plan_json(args_json)),

        // Platform plan — args_json IS the request object
        "playbackPreparePlan" => opt_json(platform_plan::playback_prepare_plan_json(args_json)),
        "libraryLocalStatePlan" => {
            opt_json(platform_plan::library_local_state_plan_json(args_json))
        }
        "preferencesSchema" => into_json(platform_plan::preferences_schema_json()),
        "applyPreferenceUpdate" => opt_json(platform_plan::apply_preference_update_json(args_json)),
        "integrationSettingsPlan" => opt_json(
            integration_settings::integration_settings_plan_json(args_json),
        ),
        "addonCollectionMutationPlan" => opt_json(
            platform_plan::addon_collection_mutation_plan_json(args_json),
        ),
        "detailEpisodePlan" => opt_json(platform_plan::detail_episode_plan_json(args_json)),
        "seasonWatchedPlan" => opt_json(platform_plan::season_watched_plan_json(args_json)),
        "markSeasonsActionPlan" => {
            opt_json(platform_plan::mark_seasons_action_plan_json(args_json))
        }
        "resourceKindToResource" => {
            let args = object(args_json)?;
            Ok(Value::String(platform_plan::resource_kind_to_resource(
                field_str(&args, "kind")?,
                args.get("requestResource").and_then(Value::as_str),
                args.get("itemResource").and_then(Value::as_str),
            )))
        }
        "parseAndPlanAddonResource" => {
            let args = object(args_json)?;
            let body = args.get("body").and_then(Value::as_str).map(str::to_string);
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?
                as i32;
            let addon_name = args
                .get("addonName")
                .and_then(Value::as_str)
                .map(str::to_string);
            let season = args.get("season").and_then(Value::as_i64);
            into_json(platform_plan::parse_and_plan_addon_resource_json(
                field_str(&args, "resource")?,
                field_str(&args, "url")?,
                status_code,
                body.as_deref(),
                field_str(&args, "kind")?,
                addon_name.as_deref(),
                season,
            ))
        }

        _ => Err(unknown_method()),
    }
}
