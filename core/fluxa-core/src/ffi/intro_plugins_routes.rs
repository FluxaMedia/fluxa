use super::*;

pub(super) fn route_intro_segments(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the data JSON for single-arg methods
        "introDbSegmentsPlan" => opt_json(intro_segments::intro_db_segments_plan_json(args_json)),
        "introDbSubmitPlan" => opt_json(intro_segments::intro_db_submit_plan_json(args_json)),
        "parseIntroDbSegments" => opt_json(intro_segments::parse_intro_db_segments_json(args_json)),
        "skipdbSegmentsPlan" => opt_json(intro_segments::skipdb_segments_plan_json(args_json)),
        "skipdbSubmitPlan" => opt_json(intro_segments::skipdb_submit_plan_json(args_json)),
        "parseSkipdbSegments" => opt_json(intro_segments::parse_skipdb_segments_json(args_json)),
        "parsePublicmetadbSegments" => {
            opt_json(intro_segments::parse_publicmetadb_segments_json(args_json))
        }
        "anilistMalId" => opt_json(intro_segments::anilist_mal_id_json(args_json)),
        "anilistId" => opt_json(intro_segments::anilist_id_json(args_json)),
        "anilistMediaIdPlan" => opt_json(intro_segments::anilist_media_id_plan_json(args_json)),
        "aniskipSegmentsPlan" => opt_json(intro_segments::aniskip_segments_plan_json(args_json)),
        "parseAniskipResults" => opt_json(intro_segments::parse_aniskip_results_json(args_json)),
        "animeSkipFindShowPlan" => {
            opt_json(intro_segments::anime_skip_find_show_plan_json(args_json))
        }
        "animeSkipShowId" => opt_json(intro_segments::anime_skip_show_id_json(args_json)),
        "animeSkipFindEpisodesPlan" => opt_json(
            intro_segments::anime_skip_find_episodes_plan_json(args_json),
        ),
        "animeSkipFindTimestampsPlan" => opt_json(
            intro_segments::anime_skip_find_timestamps_plan_json(args_json),
        ),
        "parseAnimeSkipResults" => {
            opt_json(intro_segments::parse_anime_skip_results_json(args_json))
        }
        "theIntroDbMediaPlan" => opt_json(intro_segments::the_introdb_media_plan_json(args_json)),
        "parseTheIntroDbSegments" => {
            opt_json(intro_segments::parse_the_introdb_segments_json(args_json))
        }
        "theIntroDbSubmitPlan" => opt_json(intro_segments::the_introdb_submit_plan_json(args_json)),
        "uniqueIntroSegments" => {
            let args = object(args_json)?;
            opt_json(intro_segments::unique_intro_segments_json(
                field_str(&args, "segmentsAJson")?,
                field_str(&args, "segmentsBJson")?,
            ))
        }
        "mergeIntroSegments" => opt_json(intro_segments::merge_intro_segments_json(args_json)),
        "matchAnimeSkipEpisodeId" => {
            let args = object(args_json)?;
            let season = field(&args, "season")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "season must be a number"))?;
            let episode = field(&args, "episode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "episode must be a number"))?;
            opt_json(
                intro_segments::match_anime_skip_episode_id(
                    field_str(&args, "episodesJson")?,
                    season,
                    episode,
                )
                .and_then(|id| serde_json::to_string(&id).ok()),
            )
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_plugins(method: &str, args_json: &str) -> Outcome {
    match method {
        "pluginManifestParse" => {
            let normalized = plugins::parse_plugin_manifest_json(args_json)
                .map_err(|message| fail(ErrorKind::InvalidArgs, message))?;
            into_json(normalized)
        }
        "pluginExecutionPlan" => opt_json(plugins::plugin_execution_plan_json(args_json)),
        "pluginStreamResultsParse" => {
            into_json(plugins::parse_plugin_stream_results_json(args_json))
        }
        "pluginStreamResultsToStreams" => {
            into_json(plugins::plugin_stream_results_to_streams_json(args_json))
        }

        _ => Err(unknown_method()),
    }
}
