package com.fluxa.app.shared.feature.settings

import com.fluxa.app.common.AppStrings

data class SettingsSearchEntry(val label: String, val category: SettingsCategory)

private val SETTINGS_SEARCH_KEYS: List<Pair<String, SettingsCategory>> = listOf(
    "auto.account" to SettingsCategory.Account,
    "settings.tmdb_api_key" to SettingsCategory.Account,
    "auto.disconnect" to SettingsCategory.Account,
    "settings.apis" to SettingsCategory.TmdbFeatures,
    "settings.tmdb_cast_images" to SettingsCategory.TmdbFeatures,
    "settings.tmdb_trailers" to SettingsCategory.TmdbFeatures,
    "settings.tmdb_recommendations" to SettingsCategory.TmdbFeatures,
    "settings.tmdb_ratings" to SettingsCategory.TmdbFeatures,
    "settings.notifications_title" to SettingsCategory.Notifications,
    "settings.enable_notifications" to SettingsCategory.Notifications,
    "settings.alert_new_episodes" to SettingsCategory.Notifications,
    "auto.language" to SettingsCategory.General,
    "auto.start_page" to SettingsCategory.General,
    "auto.background_playback" to SettingsCategory.General,
    "auto.accent_color" to SettingsCategory.Appearance,
    "settings.amoled" to SettingsCategory.Appearance,
    "auto.disable_animations" to SettingsCategory.Appearance,
    "settings.appearance_home_screen" to SettingsCategory.AppearanceHome,
    "auto.card_corners" to SettingsCategory.AppearanceHome,
    "auto.interface_density" to SettingsCategory.AppearanceHome,
    "auto.poster_width" to SettingsCategory.AppearanceHome,
    "settings.home_top_bar" to SettingsCategory.AppearanceHomeNavigation,
    "settings.season_posters_on_hero" to SettingsCategory.AppearanceHomeHero,
    "settings.trailer_on_home_hero" to SettingsCategory.AppearanceHomeHero,
    "auto.continue_watching" to SettingsCategory.AppearanceHomeContinueWatching,
    "settings.continue_watching_source" to SettingsCategory.AppearanceHomeContinueWatching,
    "settings.appearance_detail_screen" to SettingsCategory.AppearanceDetail,
    "settings.trailer_on_detail_hero" to SettingsCategory.AppearanceDetailHero,
    "settings.blur_unwatched_episodes" to SettingsCategory.AppearanceDetailEpisodes,
    "settings.season_selector" to SettingsCategory.AppearanceDetailEpisodes,
    "settings.episode_cards_layout" to SettingsCategory.AppearanceDetailEpisodes,
    "auto.subtitles" to SettingsCategory.Subtitles,
    "settings.advanced_settings" to SettingsCategory.Advanced,
    "auto.player" to SettingsCategory.Playback,
    "settings.anime_use_mpv" to SettingsCategory.Playback,
    "auto.playback_speed" to SettingsCategory.Playback,
    "auto.forward_rewind" to SettingsCategory.Playback,
    "settings.hold_to_speed" to SettingsCategory.Playback,
    "settings.stream_source_selection" to SettingsCategory.Playback,
    "settings.auto_play_next_episode" to SettingsCategory.Playback,
    "settings.auto_retry_next_source" to SettingsCategory.Playback,
    "settings.try_binge_group" to SettingsCategory.Playback,
    "settings.use_introdb" to SettingsCategory.Playback,
    "settings.use_aniskip" to SettingsCategory.Playback,
    "settings.use_chapter_skip" to SettingsCategory.Playback,
    "settings.preferred_audio_language" to SettingsCategory.Subtitles,
    "settings.preferred_subtitle_language" to SettingsCategory.Subtitles,
    "settings.subtitle_shadow" to SettingsCategory.Subtitles,
    "settings.subtitle_text" to SettingsCategory.Subtitles,
    "settings.subtitle_background" to SettingsCategory.Subtitles,
    "settings.subtitle_outline" to SettingsCategory.Subtitles,
    "settings.buffer_cache" to SettingsCategory.Advanced,
    "settings.forward_buffer" to SettingsCategory.Advanced,
    "settings.back_buffer" to SettingsCategory.Advanced,
    "settings.audio_decoder_mode" to SettingsCategory.Advanced,
    "settings.tunneled_playback" to SettingsCategory.Advanced,
    "settings.show_hero_section" to SettingsCategory.Content,
    "settings.hero_catalogs" to SettingsCategory.Content,
    "settings.home_catalogs" to SettingsCategory.Content,
    "settings.top_10_catalogs" to SettingsCategory.Content,
    "settings.download_source_selection" to SettingsCategory.Downloads,
    "settings.download_subtitle" to SettingsCategory.Downloads,
    "settings.automatic_updates" to SettingsCategory.Hub,
    "settings.check_for_updates" to SettingsCategory.Hub,
    "settings.developer" to SettingsCategory.Developer
)

fun settingsSearchIndex(lang: String?): List<SettingsSearchEntry> =
    SETTINGS_SEARCH_KEYS.map { (key, category) -> SettingsSearchEntry(AppStrings.t(lang, key), category) }

fun settingsSearchResults(lang: String?, query: String): List<SettingsSearchEntry> {
    if (query.isBlank()) return emptyList()
    return settingsSearchIndex(lang).filter { it.label.contains(query, ignoreCase = true) }
}
