use super::*;

pub(super) fn route_content_identity(method: &str, args_json: &str) -> Outcome {
    match method {
        "contentImdbId" => Ok(json!(content_identity::imdb_id(&arg_str(args_json, "id")?))),
        "contentBaseId" => Ok(Value::String(content_identity::base_content_id(&arg_str(
            args_json, "id",
        )?))),
        "normalizeSeriesLookupId" => Ok(Value::String(
            content_identity::normalize_series_lookup_id(&arg_str(args_json, "id")?),
        )),
        "isTmdbLikeContentId" => Ok(json!(content_identity::is_tmdb_like_content_id(&arg_str(
            args_json, "id"
        )?))),
        "tmdbNumericId" => Ok(json!(content_identity::tmdb_numeric_id(&arg_str(
            args_json, "id"
        )?))),
        "parseVideoId" => into_json(content_identity::parse_video_id_json(&arg_str(
            args_json, "id",
        )?)),
        "buildTraktIds" => opt_json(content_identity::build_trakt_ids_json(&arg_str(
            args_json, "id",
        )?)),
        "playbackIntroLookupContentId" => Ok(Value::String(
            content_identity::playback_intro_lookup_content_id(&arg_str(args_json, "id")?),
        )),
        "effectiveMetadataFeedSelection" => {
            let args = object(args_json)?;
            opt_json(content_identity::effective_metadata_feed_selection_json(
                field_str(&args, "selectedKeys")?,
                field_str(&args, "availableKeys")?,
            ))
        }
        "toggleMetadataFeedLimited" => {
            let args = object(args_json)?;
            let max_enabled = field(&args, "maxEnabled")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "maxEnabled must be a number"))?
                as i32;
            opt_json(content_identity::toggle_metadata_feed_limited_json(
                field_str(&args, "selectedKeys")?,
                field_str(&args, "availableKeys")?,
                field_str(&args, "key")?,
                max_enabled,
            ))
        }
        "streamRequestIds" => {
            let args = object(args_json)?;
            let detail_id = args.get("detailId").and_then(Value::as_str);
            let current_series_lookup_id =
                args.get("currentSeriesLookupId").and_then(Value::as_str);
            let canonical_base_id = args.get("canonicalBaseId").and_then(Value::as_str);
            Ok(json!(content_identity::stream_request_ids(
                field_str(&args, "contentType")?,
                field_str(&args, "id")?,
                detail_id,
                current_series_lookup_id,
                canonical_base_id,
            )))
        }
        "episodeTextMatches" => {
            let args = object(args_json)?;
            let season = field(&args, "season")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "season must be a number"))?
                as i32;
            let episode = field(&args, "episode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "episode must be a number"))?
                as i32;
            Ok(json!(content_identity::text_matches_episode(
                field_str(&args, "text")?,
                season,
                episode,
            )))
        }
        "streamMatchesEpisode" => {
            let args = object(args_json)?;
            let fields = [
                args.get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                args.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                args.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                args.get("filename")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                args.get("effectiveFilename")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ];
            Ok(json!(content_identity::stream_matches_episode(
                field_str(&args, "videoId")?,
                &fields,
            )))
        }
        "contentTraktKeysBatch" => opt_json(content_identity::content_trakt_keys_batch(&arg_str(
            args_json,
            "metasJson",
        )?)),
        "contentWatchedKeysBatch" => opt_json(content_identity::content_watched_keys_batch(
            &arg_str(args_json, "metasJson")?,
        )),
        "contentMergeKeys" => opt_json(content_identity::content_keys_json(
            &arg_str(args_json, "metaJson")?,
            false,
        )),
        "episodeFilenameCandidate" => {
            let args = object(args_json)?;
            opt_json(content_identity::episode_filename_candidate(
                field_str(&args, "streamJson")?,
                field_str(&args, "videoId")?,
            ))
        }
        "streamDiscoveryCacheKey" => {
            opt_str(content_identity::stream_discovery_cache_key(args_json))
        }
        "discoverCatalogCacheKey" => {
            opt_str(content_identity::discover_catalog_cache_key(args_json))
        }
        "stableFeedPart" => Ok(Value::String(content_identity::stable_feed_part(&arg_str(
            args_json, "value",
        )?))),
        "shortenSynopsis" => Ok(Value::String(content_identity::shorten_synopsis(&arg_str(
            args_json, "text",
        )?))),
        "normalizeContentType" => Ok(json!(content_identity::normalize_content_type(&arg_str(
            args_json, "value",
        )?))),
        "parseExtraArgs" => opt_json(content_identity::parse_extra_args_json(&arg_str(
            args_json, "extra",
        )?)),
        "providerSearchTerms" => Ok(json!(content_identity::provider_search_terms(&arg_str(
            args_json, "provider",
        )?))),
        "filterDiscoverResults" => {
            let args = object(args_json)?;
            let year = args.get("year").and_then(Value::as_str);
            let rating = args.get("rating").and_then(Value::as_f64).map(|v| v as f32);
            let region = args.get("region").and_then(Value::as_str);
            opt_json(content_identity::filter_discover_results_json(
                field_str(&args, "itemsJson")?,
                year,
                rating,
                region,
            ))
        }
        "directPlaybackPlan" => {
            let args = object(args_json)?;
            let detail_json = args.get("detailJson").and_then(Value::as_str);
            opt_json(content_identity::direct_playback_plan_json(
                field_str(&args, "metaJson")?,
                detail_json,
                field_str(&args, "todayIso")?,
            ))
        }
        "streamDiscoveryEpisodeContext" => {
            let args = object(args_json)?;
            let detail_json = args.get("detailJson").and_then(Value::as_str);
            opt_json(content_identity::stream_discovery_episode_context_json(
                field_str(&args, "contentType")?,
                field_str(&args, "requestId")?,
                detail_json,
                field_str(&args, "seasonEpisodesJson")?,
            ))
        }
        "parseEpisodeLocator" => {
            let raw = arg_str(args_json, "input")?;
            match content_identity::parse_episode_locator(&raw) {
                Some((base_id, season, episode)) => Ok(json!({
                    "baseId": base_id,
                    "season": season,
                    "episode": episode
                })),
                None => Ok(Value::Null),
            }
        }
        "playbackStreamRequestIds" => {
            let args = object(args_json)?;
            let detail_id = args.get("detailId").and_then(Value::as_str);
            opt_json(content_identity::playback_stream_request_ids_json(
                field_str(&args, "contentType")?,
                field_str(&args, "id")?,
                detail_id,
            ))
        }
        "toggleMetadataFeed" => {
            let args = object(args_json)?;
            opt_json(content_identity::toggle_metadata_feed_json(
                field_str(&args, "selectedKeys")?,
                field_str(&args, "availableKeys")?,
                field_str(&args, "key")?,
            ))
        }
        "setMetadataFeedGroupEnabled" => {
            let args = object(args_json)?;
            let enabled = field(&args, "enabled")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "enabled must be a bool"))?;
            opt_json(content_identity::set_metadata_feed_group_enabled_json(
                field_str(&args, "selectedKeys")?,
                field_str(&args, "availableKeys")?,
                field_str(&args, "groupKeys")?,
                enabled,
            ))
        }
        "orderedMetadataFeedKeys" => {
            let args = object(args_json)?;
            opt_json(content_identity::ordered_metadata_feed_keys(
                field_str(&args, "optionKeys")?,
                field_str(&args, "order")?,
            ))
        }
        "moveMetadataFeedOrder" => {
            let args = object(args_json)?;
            let delta = field(&args, "delta")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "delta must be a number"))?
                as i32;
            opt_json(content_identity::move_metadata_feed_order_json(
                field_str(&args, "optionKeys")?,
                field_str(&args, "currentOrder")?,
                field_str(&args, "key")?,
                delta,
            ))
        }

        _ => Err(unknown_method()),
    }
}
