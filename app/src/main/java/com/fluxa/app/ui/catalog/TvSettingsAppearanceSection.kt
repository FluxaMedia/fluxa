@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun GeneralSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.general"), AppStrings.t(lang, "auto.app")) {
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
            title = AppStrings.t(lang, "auto.start_page"),
            subtitle = AppStrings.t(lang, "settings.start_page_desc"),
            options = listOf(
                ChoiceOption("home", AppStrings.t(lang, "nav.home")),
                ChoiceOption("discover", AppStrings.t(lang, "nav.discover")),
                ChoiceOption("library", AppStrings.t(lang, "nav.library"))
            ),
            selected = profile.safeStartPage,
            onSelect = { onUpdateProfile(profile.copy(startPage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.tv_nav_layout"),
            subtitle = AppStrings.t(lang, "auto.tv_nav_layout_subtitle"),
            options = listOf(
                ChoiceOption("left", AppStrings.t(lang, "auto.tv_nav_layout_left")),
                ChoiceOption("top", AppStrings.t(lang, "auto.tv_nav_layout_top"))
            ),
            selected = profile.safeTvNavLayout,
            onSelect = { onUpdateProfile(profile.copy(tvNavLayout = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.background_playback"),
            subtitle = AppStrings.t(lang, "settings.background_playback_desc"),
            checked = profile.safeBackgroundPlayback,
            onToggle = { onUpdateProfile(profile.copy(backgroundPlayback = it)) }
        )
    }
}

@Composable
internal fun AppearanceSettings(profile: UserProfile, lang: String, previewMeta: Meta?, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.accent_color"), AppStrings.t(lang, "auto.appearance")) {
        SettingsAccentColorTile(profile, onUpdateProfile)
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.amoled"),
            subtitle = AppStrings.t(lang, "auto.makes_the_background_and_bottom_bar_pure_bla"),
            checked = profile.safeAmoledMode,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(appTheme = "dark", amoledMode = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.disable_animations"),
            subtitle = AppStrings.t(lang, "auto.turns_off_app_wide_transitions_and_card_anim"),
            checked = !profile.safeAnimationsEnabled,
            onToggle = { onUpdateProfile(profile.copy(animationsEnabled = !it)) }
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
    }
    SettingsSection(AppStrings.t(lang, "settings.appearance_home_screen"), AppStrings.t(lang, "settings.appearance_home_screen_desc")) {
        TvPosterPreview(profile, lang, previewMeta)
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.card_corners"),
            subtitle = "",
            options = mobilePosterCornerOptions(lang),
            selected = profile.safeCardCornerPreset,
            onSelect = { onUpdateProfile(profile.copy(cardCornerPreset = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.interface_density"),
            subtitle = "",
            options = mobilePresetOptions(lang),
            selected = profile.safeInterfaceDensity,
            onSelect = { onUpdateProfile(profile.copy(interfaceDensity = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.poster_width"),
            subtitle = "",
            options = mobilePosterWidthOptions(lang),
            selected = profile.safePosterWidthPreset,
            onSelect = { onUpdateProfile(profile.copy(posterWidthPreset = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.landscape_mode"),
            subtitle = "",
            checked = profile.safePosterLandscapeMode,
            onToggle = { onUpdateProfile(profile.copy(posterLandscapeMode = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.hide_titles"),
            subtitle = "",
            checked = profile.safePosterHideTitles,
            onToggle = { onUpdateProfile(profile.copy(posterHideTitles = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.season_posters_on_hero"),
            subtitle = AppStrings.t(lang, "settings.home_season_posters_on_hero_desc"),
            checked = profile.safeHomeSeasonPostersOnHero,
            onToggle = { onUpdateProfile(profile.copy(homeSeasonPostersOnHero = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.continue_watching"),
            subtitle = "",
            checked = profile.safeContinueWatchingEnabled,
            onToggle = { onUpdateProfile(profile.copy(continueWatchingEnabled = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.continue_watching_hide_titles"),
            subtitle = "",
            checked = profile.safeContinueWatchingHideTitles,
            onToggle = { onUpdateProfile(profile.copy(continueWatchingHideTitles = it)) }
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
    SettingsSection(AppStrings.t(lang, "settings.expanded_posters"), AppStrings.t(lang, "settings.expanded_posters_desc")) {
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.expanded_posters"),
            subtitle = AppStrings.t(lang, "settings.expanded_posters_desc"),
            checked = profile.safeExpandedPostersEnabled,
            onToggle = { onUpdateProfile(profile.copy(expandedPostersEnabled = it)) }
        )
        if (profile.safeExpandedPostersEnabled) {
            SettingsSecondsSliderTile(
                title = AppStrings.t(lang, "settings.expanded_posters_delay"),
                subtitle = AppStrings.t(lang, "settings.expanded_posters_delay_desc"),
                value = profile.safeExpandedPostersDelaySeconds,
                valueRange = 0..10,
                zeroLabel = AppStrings.t(lang, "settings.expanded_posters_delay_instant"),
                onValueChange = { onUpdateProfile(profile.copy(expandedPostersDelaySeconds = it)) }
            )
            SettingsToggleTile(
                title = AppStrings.t(lang, "settings.trailer_on_expanded_posters"),
                subtitle = AppStrings.t(lang, "settings.trailer_on_expanded_posters_desc"),
                checked = profile.safeTrailerOnExpandedPostersEnabled,
                onToggle = { onUpdateProfile(profile.copy(trailerOnExpandedPostersEnabled = it)) }
            )
            if (profile.safeTrailerOnExpandedPostersEnabled) {
                SettingsSecondsSliderTile(
                    title = AppStrings.t(lang, "settings.trailer_on_expanded_posters_delay"),
                    subtitle = AppStrings.t(lang, "settings.trailer_on_expanded_posters_delay_desc"),
                    value = profile.safeTrailerOnExpandedPostersDelaySeconds,
                    valueRange = 0..15,
                    onValueChange = { onUpdateProfile(profile.copy(trailerOnExpandedPostersDelaySeconds = it)) }
                )
            }
        }
    }
    SettingsSection(AppStrings.t(lang, "settings.appearance_detail_screen"), AppStrings.t(lang, "settings.appearance_detail_screen_desc")) {
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.trailer_on_hero"),
            subtitle = AppStrings.t(lang, "settings.trailer_on_hero_desc"),
            checked = profile.safeTrailerOnHero,
            onToggle = { onUpdateProfile(profile.copy(trailerOnHero = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.blur_unwatched_episodes"),
            subtitle = AppStrings.t(lang, "settings.blur_unwatched_episodes_desc"),
            checked = profile.safeBlurUnwatchedEpisodes,
            onToggle = { onUpdateProfile(profile.copy(blurUnwatchedEpisodes = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.season_selector"),
            subtitle = AppStrings.t(lang, "settings.season_selector_desc"),
            options = mobileSeasonSelectorOptions(lang),
            selected = profile.safeDetailSeasonSelectorMode,
            onSelect = { onUpdateProfile(profile.copy(detailSeasonSelectorMode = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.season_posters_on_hero"),
            subtitle = AppStrings.t(lang, "settings.detail_season_posters_on_hero_desc"),
            checked = profile.safeDetailSeasonPostersOnHero,
            onToggle = { onUpdateProfile(profile.copy(detailSeasonPostersOnHero = it)) }
        )
        TvEpisodeLayoutPreview(profile, lang) { onUpdateProfile(profile.copy(episodeCardsLayout = it)) }
    }
}
