@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun AccountSettings(
    profile: UserProfile,
    lang: String,
    onSwitchProfiles: () -> Unit,
    onConnectTrakt: () -> Unit,
    onConnectMal: () -> Unit,
    onConnectSimkl: () -> Unit,
    onUpdateProfile: (UserProfile) -> Unit
) {
    var tmdbExpanded by remember { mutableStateOf(false) }
    SettingsSection(
        AppStrings.t(lang, "auto.account_sync"),
        AppStrings.t(lang, "auto.account_devices_and_sync")
    ) {
        SettingsActionTile(
            title = AppStrings.t(lang, "settings.switch_profiles"),
            subtitle = AppStrings.t(lang, "settings.switch_profiles_desc"),
            icon = FluxaIcons.AccountCircle,
            onClick = onSwitchProfiles
        )
    }
    SettingsSection(
        AppStrings.t(lang, "settings.sync_with"),
        AppStrings.t(lang, "settings.sync_with_desc")
    ) {
        SettingsConnectionTile(
            title = AppStrings.t(lang, "brand.trakt"),
            iconRes = R.drawable.ic_trakt,
            value = if (!profile.traktAccessToken.isNullOrBlank()) {
                AppStrings.t(lang, "auto.connected")
            } else {
                AppStrings.t(lang, "auto.not_connected")
            },
            onClick = onConnectTrakt
        )
        SettingsConnectionTile(
            title = AppStrings.t(lang, "brand.myanimelist"),
            iconRes = R.drawable.ic_myanimelist,
            value = if (!profile.malAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.connected") else AppStrings.t(lang, "auto.not_connected"),
            onClick = onConnectMal
        )
        SettingsConnectionTile(
            title = AppStrings.t(lang, "brand.simkl"),
            iconRes = R.drawable.ic_simkl,
            value = if (!profile.simklAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.connected") else AppStrings.t(lang, "auto.not_connected"),
            onClick = onConnectSimkl
        )
        if (!profile.traktAccessToken.isNullOrBlank() || !profile.malAccessToken.isNullOrBlank() || !profile.simklAccessToken.isNullOrBlank()) {
            SettingsActionTile(
                title = AppStrings.t(lang, "auto.disconnect"),
                subtitle = AppStrings.t(lang, "integration.connected_accounts"),
                icon = FluxaIcons.Logout,
                accent = Color(0xFFFF6B6B),
                onClick = {
                    onUpdateProfile(
                        profile.copy(
                            traktAccessToken = null,
                            traktRefreshToken = null,
                            traktLastSyncAt = null,
                            traktLastSyncedItems = null,
                            traktLastContinueWatchingCount = null,
                            traktLastWatchlistCount = null,
                            malAccessToken = null,
                            malRefreshToken = null,
                            simklAccessToken = null
                        )
                    )
                }
            )
        }
    }
    SettingsSection(
        AppStrings.t(lang, "settings.apis"),
        AppStrings.t(lang, "settings.apis_desc")
    ) {
        SettingsActionTile(
            title = AppStrings.t(lang, "settings.tmdb_api"),
            subtitle = if (profile.safeTmdbApiKey.isNotBlank()) AppStrings.t(lang, "settings.tmdb_api_configured") else AppStrings.t(lang, "settings.tmdb_api_not_configured"),
            icon = FluxaIcons.Storage,
            onClick = { tmdbExpanded = !tmdbExpanded }
        )
        if (tmdbExpanded) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.tmdb_api_key"),
                subtitle = AppStrings.t(lang, "settings.tmdb_api_key_desc"),
                value = profile.safeTmdbApiKey,
                placeholder = AppStrings.t(lang, "settings.tmdb_api_key_placeholder"),
                onValueChange = { onUpdateProfile(profile.copy(tmdbApiKey = it.trim())) }
            )
            if (profile.safeTmdbApiKey.isNotBlank()) {
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_cast_images"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_cast_images_desc"),
                    checked = profile.safeTmdbCastImagesEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbCastImagesEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_similar_results"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_similar_results_desc"),
                    checked = profile.safeTmdbSimilarResultsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbSimilarResultsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_trailers"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_trailers_desc"),
                    checked = profile.safeTmdbTrailersEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbTrailersEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_recommendations"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_recommendations_desc"),
                    checked = profile.safeTmdbRecommendationsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbRecommendationsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_collection_info"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_collection_info_desc"),
                    checked = profile.safeTmdbCollectionInfoEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbCollectionInfoEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_episode_images"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_episode_images_desc"),
                    checked = profile.safeTmdbEpisodeImagesEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbEpisodeImagesEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_logos_backdrops"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_logos_backdrops_desc"),
                    checked = profile.safeTmdbLogosBackdropsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbLogosBackdropsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_ratings"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_ratings_desc"),
                    checked = profile.safeTmdbRatingsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbRatingsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_basic_info"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_basic_info_desc"),
                    checked = profile.safeTmdbBasicInfoEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbBasicInfoEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_details"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_details_desc"),
                    checked = profile.safeTmdbDetailsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbDetailsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_productions"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_productions_desc"),
                    checked = profile.safeTmdbProductionsEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbProductionsEnabled = it)) }
                )
                SettingsToggleTile(
                    title = AppStrings.t(lang, "settings.tmdb_networks"),
                    subtitle = AppStrings.t(lang, "settings.tmdb_networks_desc"),
                    checked = profile.safeTmdbNetworksEnabled,
                    onToggle = { onUpdateProfile(profile.copy(tmdbNetworksEnabled = it)) }
                )
            }
        }
    }
}

@Composable
internal fun InterfaceSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.interface_c0c2eda7"), AppStrings.t(lang, "auto.tune_language_and_visual_layout")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.interface_language"),
            subtitle = AppStrings.t(lang, "auto.app_copy_and_category_labels"),
            options = listOf(
                ChoiceOption("none", AppStrings.t(lang, "settings.none")),
                ChoiceOption("tr", languageDisplayName("tr", lang)),
                ChoiceOption("en", languageDisplayName("en", lang))
            ),
            selected = profile.safeLanguage,
            onSelect = { onUpdateProfile(profile.copy(language = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.card_layout"),
            subtitle = AppStrings.t(lang, "auto.how_rows_and_home_shelves_look"),
            options = listOf(
                ChoiceOption("vertical", AppStrings.t(lang, "auto.vertical_layout")),
                ChoiceOption("horizontal", AppStrings.t(lang, "auto.horizontal"))
            ),
            selected = profile.safeCardLayout,
            onSelect = { onUpdateProfile(profile.copy(cardLayout = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.continue_watching_layout"),
            subtitle = AppStrings.t(lang, "auto.show_that_shelf_as_posters_or_episode_cards"),
            options = listOf(
                ChoiceOption("vertical", AppStrings.t(lang, "auto.vertical_layout")),
                ChoiceOption("horizontal", AppStrings.t(lang, "auto.horizontal")),
                ChoiceOption("inherit", AppStrings.t(lang, "auto.match_global"))
            ),
            selected = profile.safeContinueWatchingLayout,
            onSelect = { onUpdateProfile(profile.copy(continueWatchingLayout = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.series_artwork"),
            subtitle = AppStrings.t(lang, "auto.choose_episode_stills_or_regular_artwork_for"),
            options = listOf(
                ChoiceOption("episode", AppStrings.t(lang, "auto.episode_cover")),
                ChoiceOption("poster", AppStrings.t(lang, "auto.poster")),
                ChoiceOption("background", AppStrings.t(lang, "auto.backdrop"))
            ),
            selected = profile.safeContinueWatchingArtwork,
            onSelect = { onUpdateProfile(profile.copy(continueWatchingArtwork = it)) }
        )
    }
}

@Composable
internal fun PlaybackSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.playback"), AppStrings.t(lang, "auto.shape_how_playback_behaves")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.preferred_audio_language"),
            subtitle = AppStrings.t(lang, "auto.default_audio_track_preference"),
            options = audioLanguageOptions(lang),
            selected = profile.safePreferredAudioLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredAudioLanguage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.skip_forward"),
            subtitle = AppStrings.t(lang, "auto.how_far_right_arrow_should_jump"),
            options = listOf(ChoiceOption("10", "10s"), ChoiceOption("15", "15s"), ChoiceOption("30", "30s")),
            selected = profile.safeSeekForwardSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(seekForwardSeconds = it.toInt())) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.skip_back"),
            subtitle = AppStrings.t(lang, "auto.how_far_left_arrow_should_rewind"),
            options = listOf(ChoiceOption("10", "10s"), ChoiceOption("15", "15s"), ChoiceOption("30", "30s")),
            selected = profile.safeSeekBackwardSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(seekBackwardSeconds = it.toInt())) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.auto_skip_intros"),
            subtitle = AppStrings.t(lang, "auto.prefer_skipping_intros_where_supported"),
            checked = profile.safeAutoSkipIntro,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(autoSkipIntro = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.stream_source_selection"),
            subtitle = AppStrings.t(lang, "settings.stream_source_selection_desc"),
            options = streamSourceSelectionOptions(lang),
            selected = profile.safeStreamSourceSelectionMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(streamSourceSelectionMode = it)) }
        )
        if (profile.safeStreamSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.regex_pattern"),
                subtitle = AppStrings.t(lang, "settings.regex_pattern_desc"),
                value = profile.safeStreamSourceRegexPattern,
                placeholder = AppStrings.t(lang, "settings.regex_pattern_placeholder"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(streamSourceRegexPattern = it)) }
            )
        }
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.auto_play_next_episode"),
            subtitle = AppStrings.t(lang, "auto.queue_the_next_episode_automatically"),
            checked = profile.safeAutoPlayNextEpisode,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(autoPlayNextEpisode = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.try_binge_group"),
            subtitle = AppStrings.t(lang, "settings.try_binge_group_desc"),
            checked = profile.safeTryBingeGroup,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(tryBingeGroup = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.preferred_player"),
            subtitle = AppStrings.t(lang, "auto.choose_between_internal_or_external_playback"),
            options = listOf(
                ChoiceOption("exoplayer", "ExoPlayer"),
                ChoiceOption("mpv", "MPV")
            ),
            selected = profile.safePreferredPlayer,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredPlayer = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.dv_fallback"),
            subtitle = AppStrings.t(lang, "settings.dv_fallback_desc"),
            options = dolbyVisionFallbackOptions(lang),
            selected = profile.safeDolbyVisionFallbackMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(dolbyVisionFallbackMode = it)) }
        )
        if (profile.safeDolbyVisionFallbackMode == "convert_dv81") {
            SettingsChoiceTile(
                title = AppStrings.t(lang, "settings.dv_rpu_mode"),
                subtitle = AppStrings.t(lang, "settings.dv_rpu_mode_desc"),
                options = dvRpuModeOptions(lang),
                selected = profile.safeDvRpuMode.toString(),
                onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvRpuMode = it.toIntOrNull() ?: 2)) }
            )
            SettingsToggleTile(
                title = AppStrings.t(lang, "settings.dv_zero_level5"),
                subtitle = AppStrings.t(lang, "settings.dv_zero_level5_desc"),
                checked = profile.safeDvZeroLevel5,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(dvZeroLevel5 = it)) }
            )
            SettingsChoiceTile(
                title = AppStrings.t(lang, "settings.dv_hdr10plus_mode"),
                subtitle = AppStrings.t(lang, "settings.dv_hdr10plus_mode_desc"),
                options = dvHdr10PlusModeOptions(lang),
                selected = profile.safeDvHdr10PlusMode,
                onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvHdr10PlusMode = it)) }
            )
        }
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.audio_decoder_mode"),
            subtitle = AppStrings.t(lang, "settings.audio_decoder_mode_desc"),
            options = audioDecoderModeOptions(lang),
            selected = profile.safeAudioDecoderMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(audioDecoderMode = it)) }
        )
    }
    SubtitleSettings(profile, lang, onUpdateProfile)
}

@Composable
internal fun DownloadSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.downloads"), AppStrings.t(lang, "settings.downloads_desc")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.download_source_selection"),
            subtitle = AppStrings.t(lang, "settings.download_source_selection_desc"),
            options = streamSourceSelectionOptions(lang),
            selected = profile.safeDownloadSourceSelectionMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceSelectionMode = it)) }
        )
        if (profile.safeDownloadSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.regex_pattern"),
                subtitle = AppStrings.t(lang, "settings.regex_pattern_desc"),
                value = profile.safeDownloadSourceRegexPattern,
                placeholder = AppStrings.t(lang, "settings.regex_pattern_placeholder"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceRegexPattern = it)) }
            )
        }
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.download_subtitle"),
            subtitle = AppStrings.t(lang, "settings.download_subtitle_desc"),
            options = downloadSubtitleOptions(lang),
            selected = profile.safeDownloadSubtitleLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSubtitleLanguage = it)) }
        )
    }
}

@Composable
internal fun SubtitleSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.subtitles_fc449c82"), AppStrings.t(lang, "auto.subtitle_language_size_and_readability")) {
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.auto_enable_subtitles"),
            subtitle = AppStrings.t(lang, "auto.enable_subtitles_automatically_when_availabl"),
            checked = profile.safeAutoEnableSubtitles,
            onToggle = { onUpdateProfile(profile.copy(autoEnableSubtitles = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.preferred_subtitle_language"),
            subtitle = AppStrings.t(lang, "auto.default_subtitle_track_preference"),
            options = subtitleLanguageOptions(lang),
            selected = profile.safePreferredSubtitleLanguage,
            onSelect = { onUpdateProfile(profile.copy(preferredSubtitleLanguage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.subtitle_size"),
            subtitle = AppStrings.t(lang, "auto.tune_readability_on_tv_and_mobile"),
            options = listOf(
                ChoiceOption("18", "18"),
                ChoiceOption("20", "20"),
                ChoiceOption("24", "24"),
                ChoiceOption("28", "28")
            ),
            selected = profile.safeSubtitleSize.toInt().toString(),
            onSelect = { onUpdateProfile(profile.copy(subtitleSize = it.toFloat())) }
        )
        // Preview Background
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 16.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(Color.Black.copy(alpha = 0.6f))
                .padding(24.dp),
            contentAlignment = Alignment.Center
        ) {
            SubtitlePreviewText(
                text = AppStrings.t(lang, "settings.subtitle_preview"),
                profile = profile
            )
        }
    }
}

@Composable
internal fun AddonSettings(profile: UserProfile, lang: String, onManageAddons: () -> Unit, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.add_ons_and_torrent"), AppStrings.t(lang, "auto.sources_and_stream_engine_preferences")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.torrent_speed"),
            subtitle = AppStrings.t(lang, "auto.stremio_like_speed_presets"),
            options = listOf(
                ChoiceOption("default", AppStrings.t(lang, "auto.default")),
                ChoiceOption("fast", AppStrings.t(lang, "auto.fast")),
                ChoiceOption("ultra_fast", AppStrings.t(lang, "auto.ultra_fast"))
            ),
            selected = profile.safeTorrentSpeedPreset,
            onSelect = { onUpdateProfile(profile.copy(torrentSpeedPreset = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.cache_size"),
            subtitle = AppStrings.t(lang, "auto.torrent_cache_budget"),
            options = listOf(
                ChoiceOption("2gb", "2 GB"),
                ChoiceOption("5gb", "5 GB"),
                ChoiceOption("10gb", "10 GB"),
                ChoiceOption("unlimited", AppStrings.t(lang, "auto.unlimited"))
            ),
            selected = profile.safeTorrentCachePreset,
            onSelect = { onUpdateProfile(profile.copy(torrentCachePreset = it)) }
        )
        SettingsInfoTile(
            title = AppStrings.t(lang, "auto.streaming_server"),
            value = profile.safeStreamingServerUrl,
            icon = FluxaIcons.Memory
        )
    }
}

@Composable
internal fun SystemSettings(profile: UserProfile, lang: String, onReboot: () -> Unit, onLogout: () -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.system"), AppStrings.t(lang, "auto.app_and_device_actions")) {
        SettingsInfoTile(AppStrings.t(lang, "auto.version"), "1.2.5", FluxaIcons.Settings)
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.restart_app"),
            subtitle = AppStrings.t(lang, "auto.refresh_the_app_after_playback_or_network_ch"),
            icon = FluxaIcons.Settings,
            onClick = onReboot
        )
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.log_out"),
            subtitle = AppStrings.t(lang, "auto.sign_out_from_the_active_profile_on_this_dev"),
            icon = FluxaIcons.Logout,
            accent = Color(0xFFFF6B6B),
            onClick = onLogout
        )
    }
}
