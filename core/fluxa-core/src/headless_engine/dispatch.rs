use super::HeadlessEngine;
use super::contracts::{AppAction, SourceSelectionMode};
use super::{
    addons, auth, calendar, detail, discover, home, library, navigation, offline, player, plugins,
    search, settings, sync, trailer,
};
use crate::runtime::EffectEnvelope;
use serde::Serialize;
use serde_json::Value;

fn wire_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn source_selection_mode_value(mode: Option<SourceSelectionMode>) -> Option<String> {
    mode.map(|value| value.as_str().to_string())
}

impl HeadlessEngine {
    pub(super) fn dispatch(&mut self, action: AppAction) -> Vec<EffectEnvelope> {
        match action {
            AppAction::NavigationRequested { route, params } => {
                navigation::dispatch(self, route, params)
            }
            AppAction::DetailLoadRequested {
                content_type,
                id,
                language,
                source_addon_transport_url,
                source_addon_catalog_type,
                profile,
            } => detail::dispatch_load(
                self,
                content_type,
                id,
                language,
                source_addon_transport_url,
                source_addon_catalog_type,
                profile,
            ),
            AppAction::DetailLocalStateRequested {
                primary_id,
                fallback_id,
                content_type,
                profile,
            } => detail::dispatch_local_state(self, primary_id, fallback_id, content_type, profile),
            AppAction::DetailSecondaryRequested {
                content_type,
                id,
                language,
                profile,
                similar_titles_source,
            } => detail::dispatch_secondary(
                self,
                content_type,
                id,
                language,
                profile,
                similar_titles_source,
            ),
            AppAction::DetailPrefetchRequested {
                content_type,
                id,
                stream_lookup_id,
                title,
                original_name,
                year,
                language,
                profile,
            } => detail::dispatch_prefetch(
                self,
                content_type,
                id,
                stream_lookup_id,
                title,
                original_name,
                year,
                language,
                profile,
            ),
            AppAction::DetailStreamsRequested {
                content_type,
                request_ids,
                detail,
                season_episodes,
                language,
                profile,
            } => detail::dispatch_streams(
                self,
                content_type,
                request_ids,
                detail.map(wire_value),
                season_episodes.map(|items| items.into_iter().map(wire_value).collect()),
                language,
                profile.map(wire_value),
            ),
            AppAction::DetailStreamsAppended {
                streams,
                available_addons,
                generation,
            } => detail::dispatch_streams_appended(
                self,
                streams.into_iter().map(wire_value).collect(),
                available_addons,
                generation,
            ),
            AppAction::DetailSelectedAddonChanged { addon } => {
                detail::dispatch_selected_addon_changed(self, addon)
            }
            AppAction::MetaDetailRequested {
                content_type,
                id,
                language,
                profile,
            } => detail::dispatch_meta_detail(self, content_type, id, language, profile),
            AppAction::DirectPlaybackRequested {
                meta,
                language,
                profile,
            } => home::dispatch_direct_playback(self, meta, language, profile),
            AppAction::IntroSegmentsRequested {
                imdb_id,
                season,
                episode,
                title,
                use_intro_db,
                use_ani_skip,
            } => player::dispatch_intro_segments(
                self,
                imdb_id,
                season,
                episode,
                title,
                use_intro_db,
                use_ani_skip,
            ),
            AppAction::IntroImdbIdRequested {
                meta,
                video_id,
                language,
            } => player::dispatch_intro_imdb_id(self, meta, video_id, language),
            AppAction::PlayerLoadStreamsRequested {
                content_type,
                id,
                current_video_id,
                initial_video_id,
                initial_streams,
                initial_stream_index,
                saved_url,
                saved_title,
                source_selection_mode,
                regex_pattern,
                preferred_binge_group,
                title,
                original_name,
                year,
                language,
                profile,
                outgoing_progress,
            } => player::dispatch_load_streams(
                self,
                content_type,
                id,
                current_video_id,
                initial_video_id,
                initial_streams.map(|items| items.into_iter().map(wire_value).collect()),
                initial_stream_index,
                saved_url,
                saved_title,
                source_selection_mode_value(source_selection_mode),
                regex_pattern,
                preferred_binge_group,
                title,
                original_name,
                year,
                language,
                profile.map(wire_value),
                outgoing_progress,
            ),
            AppAction::PlayerStreamsLoaded {
                streams,
                current_video_id,
                initial_stream_index,
                saved_url,
                saved_title,
                source_selection_mode,
                regex_pattern,
                preferred_binge_group,
            } => player::dispatch_streams_loaded(
                self,
                streams.into_iter().map(wire_value).collect(),
                current_video_id,
                initial_stream_index,
                saved_url,
                saved_title,
                source_selection_mode_value(source_selection_mode),
                regex_pattern,
                preferred_binge_group,
            ),
            AppAction::PlayerStreamsFailed { error_code } => {
                player::dispatch_streams_failed(self, error_code)
            }
            AppAction::PlayerResolvePlaybackRequested {
                url,
                stream,
                current_video_id,
                title,
            } => player::dispatch_resolve_playback(
                self,
                url,
                stream.map(wire_value),
                current_video_id,
                title,
            ),
            AppAction::ScrobbleRequested {
                token,
                meta_type,
                item_id,
                progress,
                action_name,
                profile,
            } => player::dispatch_scrobble(
                self,
                token,
                meta_type,
                item_id,
                progress,
                action_name,
                profile,
            ),
            AppAction::ProfileActivated { profile } => {
                library::dispatch_profile_activated(self, profile)
            }
            AppAction::HomeLoadRequested {
                profile,
                language,
                force,
            } => home::dispatch_load(self, profile, language, force),
            AppAction::RefreshContinueWatchingRequested {
                profile,
                language,
                source,
            } => home::dispatch_refresh_continue_watching(self, profile, language, source),
            AppAction::LibraryHydrateRequested { profile_id } => {
                library::dispatch_hydrate(self, profile_id)
            }
            AppAction::ToggleWatchlistRequested { item, profile } => {
                library::dispatch_toggle_watchlist(self, item, profile)
            }
            AppAction::ToggleLibraryStatusRequested { list, item } => {
                library::dispatch_toggle_status(self, list, item)
            }
            AppAction::SetFeedbackRequested { id, value, meta } => {
                library::dispatch_set_feedback(self, id, value, meta)
            }
            AppAction::ClearPlaybackProgressRequested { profile, meta } => {
                library::dispatch_clear_progress(self, profile, meta)
            }
            AppAction::SavePlaybackProgressRequested { action } => library::dispatch_save_progress(
                self,
                action.profile,
                action.meta,
                action.time_offset,
                action.duration,
                action.last_video_id,
                action.last_stream_index,
                action.last_episode_name,
                action.last_episode_season,
                action.last_episode_number,
                action.last_episode_thumbnail,
                action.last_stream_url,
                action.last_stream_title,
                action.last_audio_language,
                action.last_subtitle_language,
                action.scrobble_trakt_pause,
                action.refresh_external_continue_watching,
            ),
            AppAction::MarkWatchedRequested { action } => library::dispatch_mark_watched(
                self,
                action.series_id,
                action.video_ids,
                action.watched,
                action.meta,
                action.episodes,
                action.profile,
            ),
            AppAction::AddonInstallRequested {
                transport_url,
                force_refresh,
            } => addons::dispatch_install(self, transport_url, force_refresh),
            AppAction::AddonsRefreshRequested {
                profile,
                force_refresh,
            } => addons::dispatch_refresh(self, profile, force_refresh),
            AppAction::AddonResourceRequested {
                transport_url,
                resource,
                content_type,
                id,
                extra,
            } => addons::dispatch_resource(self, transport_url, resource, content_type, id, extra),
            AppAction::SearchRequested {
                query,
                profile,
                language,
            } => search::dispatch(self, query, profile, language),
            AppAction::DiscoverRequested {
                content_type,
                filters,
                profile,
                language,
            } => discover::dispatch_discover(self, content_type, filters, profile, language),
            AppAction::DiscoverCatalogFiltersRequested {
                content_type,
                selected_catalog_key,
                profile,
                language,
            } => discover::dispatch_catalog_filters(
                self,
                content_type,
                selected_catalog_key,
                profile,
                language,
            ),
            AppAction::DiscoverPageRequested {
                transport_url,
                content_type,
                catalog_id,
                skip,
                genre,
            } => discover::dispatch_discover_page(
                self,
                transport_url,
                content_type,
                catalog_id,
                skip,
                genre,
            ),
            AppAction::CatalogPageRequested {
                category_id,
                transport_url,
                content_type,
                catalog_id,
                skip,
                genre,
                search,
                remote_source,
                profile,
            } => home::dispatch_catalog_page(
                self,
                category_id,
                transport_url,
                content_type,
                catalog_id,
                skip,
                genre,
                search,
                remote_source,
                profile,
            ),
            AppAction::DetailSeasonRequested {
                series_id,
                season,
                profile,
                language,
            } => detail::dispatch_season(self, series_id, season, profile, language),
            AppAction::PlayerNextEpisodeCardShown {
                content_type,
                series_id,
                next_video_id,
                title,
                original_name,
                year,
                language,
                profile,
            } => player::dispatch_next_episode_prefetch(
                self,
                content_type,
                series_id,
                next_video_id,
                title,
                original_name,
                year,
                language,
                profile,
            ),
            AppAction::SubtitleLoadRequested {
                stream,
                content_type,
                id,
                extra_args,
            } => player::dispatch_subtitle_load(self, stream, content_type, id, extra_args),
            AppAction::ExternalSyncRequested {
                provider,
                profile,
                language,
            } => sync::dispatch_external_sync(self, provider, profile, language),
            AppAction::AuthFlowRequested { provider, mode } => {
                auth::dispatch_flow(self, provider, mode)
            }
            AppAction::AuthExchangeRequested {
                provider,
                code,
                code_verifier,
                profile,
            } => auth::dispatch_exchange(self, provider, code, code_verifier, profile),
            AppAction::AuthRefreshRequested { provider, profile } => {
                auth::dispatch_token_refresh(self, provider, profile)
            }
            AppAction::ExternalIntegrationSyncRequested {
                provider,
                profile,
                language,
            } => sync::dispatch_integration_sync(self, provider, profile, language),
            AppAction::SettingsChanged { key, value } => settings::dispatch(self, key, value),
            AppAction::CalendarMonthRequested {
                profile,
                year,
                month,
                planned_items,
            } => calendar::dispatch(self, profile, year, month, planned_items),
            AppAction::OfflineDownloadRequested {
                meta,
                stream,
                video_id,
                video,
                subtitle,
                profile_id,
                language,
            } => offline::dispatch(
                self, meta, stream, video_id, video, subtitle, profile_id, language,
            ),
            AppAction::TrailerResolveRequested {
                request_id,
                video_id,
                max_height,
            } => trailer::dispatch_resolve(self, request_id, video_id, max_height),
            AppAction::TrailerPrewarmRequested => trailer::dispatch_prewarm(self),
            AppAction::PluginRepositoryAddRequested { manifest_url } => {
                plugins::dispatch_add_repository(self, manifest_url)
            }
            AppAction::PluginRepositoryRemoveRequested { manifest_url } => {
                plugins::dispatch_remove_repository(self, manifest_url)
            }
            AppAction::PluginScraperToggled {
                scraper_id,
                enabled,
            } => plugins::dispatch_toggle_scraper(self, scraper_id, enabled),
            AppAction::PluginScraperSettingsUpdated {
                scraper_id,
                settings,
            } => plugins::dispatch_update_scraper_settings(self, scraper_id, settings),
        }
    }
}
