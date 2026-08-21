package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaThemePacks
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Composable
internal fun SettingsGeneralContent(model: SettingsGeneralUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val languageOptions = listOf(
        SettingsChoiceOption("en", AppStrings.t(lang, "language.english")),
        SettingsChoiceOption("tr", AppStrings.t(lang, "language.turkish"))
    )
    val startPageOptions = listOf(
        SettingsChoiceOption("home", AppStrings.t(lang, "nav.home")),
        SettingsChoiceOption("discover", AppStrings.t(lang, "nav.discover")),
        SettingsChoiceOption("library", AppStrings.t(lang, "nav.library"))
    )
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.language"), model.language, languageOptions) { onAction(SettingsAction.GeneralChanged(model.copy(language = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.start_page"), model.startPage, startPageOptions) { onAction(SettingsAction.GeneralChanged(model.copy(startPage = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.background_playback"), description = AppStrings.t(lang, "settings.background_playback_desc"), value = model.backgroundPlayback) {
            onAction(SettingsAction.GeneralChanged(model.copy(backgroundPlayback = it)))
        }
    }
}

@Composable
internal fun SettingsAppearanceContent(model: SettingsAppearanceUiModel, lang: String?, onAction: (SettingsAction) -> Unit, onNavigate: (SettingsCategory) -> Unit) {
    val themeOptions = listOf(
        SettingsChoiceOption("fluxa-dark", AppStrings.t(lang, "theme.fluxa_dark")),
        SettingsChoiceOption("amoled", AppStrings.t(lang, "theme.amoled")),
        SettingsChoiceOption("midnight-blue", AppStrings.t(lang, "theme.midnight_blue")),
    )
    val themeJson = Json { encodeDefaults = true }
    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_theme"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.theme"), model.themeId, themeOptions) { id ->
            val pack = when (id) {
                "amoled" -> FluxaThemePacks.fluxaDark.copy(id = "amoled", nameKey = "theme.amoled", colors = FluxaThemePacks.fluxaDark.colors.copy(background = "#000000", backgroundElevated = "#000000", surface = "#080808", surfaceRaised = "#141414", navigation = "#000000"))
                "midnight-blue" -> FluxaThemePacks.fluxaDark.copy(id = "midnight-blue", nameKey = "theme.midnight_blue", colors = FluxaThemePacks.fluxaDark.colors.copy(background = "#080A10", backgroundElevated = "#0D1220", surface = "#121A2A", surfaceRaised = "#1B263D", navigation = "#090E1A", textSecondary = "#A9B4C8", textMuted = "#71809A", accent = "#5C8DFF"))
                else -> FluxaThemePacks.fluxaDark
            }
            onAction(SettingsAction.AppearanceChanged(model.copy(themeId = id, themeJson = themeJson.encodeToString(pack))))
        }
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 10.dp)) {
            Text(AppStrings.t(lang, "auto.accent_color"), color = Color.White, modifier = Modifier.weight(1f))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SETTINGS_COLOR_SWATCHES.forEach { swatch ->
                    var swatchFocused by remember { mutableStateOf(false) }
                    val selected = swatch == model.accentColorArgb
                    Box(
                        modifier = Modifier
                            .size(24.dp)
                            .clip(CircleShape)
                            .onFocusChanged { swatchFocused = it.isFocused }
                            .background(Color(swatch.toInt()))
                            .then(
                                if (selected || swatchFocused) Modifier.border(2.dp, Color.White, CircleShape) else Modifier
                            )
                            .clickable { onAction(SettingsAction.AppearanceChanged(model.copy(accentColorArgb = swatch))) },
                        contentAlignment = Alignment.Center
                    ) {
                        if (selected) {
                            val isLight = Color(swatch.toInt()).luminance() > 0.5f
                            Icon(
                                Icons.Filled.CheckCircle,
                                contentDescription = null,
                                tint = if (isLight) Color.Black else Color.White,
                                modifier = Modifier.size(14.dp)
                            )
                        }
                    }
                }
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.amoled"), value = model.amoledMode) { onAction(SettingsAction.AppearanceChanged(model.copy(amoledMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.liquid_glass"), description = AppStrings.t(lang, "settings.liquid_glass_desc"), value = model.liquidGlassMode) {
            onAction(SettingsAction.AppearanceChanged(model.copy(liquidGlassMode = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_layout"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "auto.disable_animations"), value = !model.animationsEnabled) {
            onAction(SettingsAction.AppearanceChanged(model.copy(animationsEnabled = !it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.floating_bottom_bar"), value = model.floatingBottomBar) {
            onAction(SettingsAction.AppearanceChanged(model.copy(floatingBottomBar = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.bottom_bar_labels"), value = model.bottomBarLabels) {
            onAction(SettingsAction.AppearanceChanged(model.copy(bottomBarLabels = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.top_navigation_bar"), description = AppStrings.t(lang, "settings.top_navigation_bar_desc"), value = model.topNavigationBar) {
            onAction(SettingsAction.AppearanceChanged(model.copy(topNavigationBar = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_screens"))
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "settings.appearance_home_screen"),
            description = AppStrings.t(lang, "settings.appearance_home_screen_desc")
        ) { onNavigate(SettingsCategory.AppearanceHome) }
        SettingsNavRow(
            AppStrings.t(lang, "settings.appearance_detail_screen"),
            description = AppStrings.t(lang, "settings.appearance_detail_screen_desc")
        ) { onNavigate(SettingsCategory.AppearanceDetail) }
    }
}

@Composable
internal fun SettingsAppearanceHomeContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val cornerOptions = listOf(
        SettingsChoiceOption("sharp", AppStrings.t(lang, "auto.sharp")),
        SettingsChoiceOption("classic", AppStrings.t(lang, "auto.classic")),
        SettingsChoiceOption("soft", AppStrings.t(lang, "auto.soft")),
        SettingsChoiceOption("rounded", AppStrings.t(lang, "auto.rounded")),
        SettingsChoiceOption("pill", AppStrings.t(lang, "auto.extra_rounded"))
    )
    val densityOptions = listOf(
        SettingsChoiceOption("small", AppStrings.t(lang, "auto.small")),
        SettingsChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        SettingsChoiceOption("large", AppStrings.t(lang, "auto.large"))
    )
    val posterWidthOptions = listOf(
        SettingsChoiceOption("xsmall", AppStrings.t(lang, "auto.very_small")),
        SettingsChoiceOption("small", AppStrings.t(lang, "auto.small")),
        SettingsChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        SettingsChoiceOption("large", AppStrings.t(lang, "auto.large")),
        SettingsChoiceOption("xlarge", AppStrings.t(lang, "auto.very_large"))
    )
    SettingsPosterPreview(model)
    SettingsSectionHeader(AppStrings.t(lang, "settings.layout"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.card_corners"), model.cardCornerPreset, cornerOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(cardCornerPreset = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.interface_density"), model.interfaceDensity, densityOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(interfaceDensity = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.poster_width"), model.posterWidthPreset, posterWidthOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterWidthPreset = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.landscape_mode"), value = model.posterLandscapeMode) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterLandscapeMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.hide_titles"), value = model.posterHideTitles) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterHideTitles = it))) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_banner"))
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.season_posters_on_hero"),
            description = AppStrings.t(lang, "settings.home_season_posters_on_hero_desc"),
            value = model.homeSeasonPostersOnHero
        ) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(homeSeasonPostersOnHero = it))) }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.trailer_on_home_hero"),
            description = AppStrings.t(lang, "settings.trailer_on_home_hero_desc"),
            value = model.trailerOnHomeHeroEnabled
        ) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(trailerOnHomeHeroEnabled = it))) }
        if (model.trailerOnHomeHeroEnabled) {
            SettingsStepperRow(AppStrings.t(lang, "settings.trailer_on_home_hero_delay"), model.trailerOnHomeHeroDelaySeconds, min = 0, max = 15, formatValue = { "${it}s" }) {
                onAction(SettingsAction.AppearanceHomeChanged(model.copy(trailerOnHomeHeroDelaySeconds = it)))
            }
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "auto.continue_watching"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "auto.continue_watching"), value = model.continueWatchingEnabled) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingEnabled = it)))
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.upcoming_row"),
            description = AppStrings.t(lang, "settings.upcoming_row_desc"),
            value = model.upcomingRowEnabled
        ) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(upcomingRowEnabled = it)))
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.continue_watching_horizontal"),
            description = AppStrings.t(lang, "settings.continue_watching_horizontal_desc"),
            value = model.continueWatchingHorizontal
        ) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHorizontal = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.continue_watching_hide_titles"), value = model.continueWatchingHideTitles) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHideTitles = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.card_corners"), model.continueWatchingCardCornerPreset, cornerOptions) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingCardCornerPreset = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.interface_density"), model.continueWatchingInterfaceDensity, densityOptions) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingInterfaceDensity = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.poster_width"), model.continueWatchingWidthPreset, posterWidthOptions) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingWidthPreset = it)))
        }
    }
}

internal fun posterCornerRadius(preset: String): androidx.compose.ui.unit.Dp = com.fluxa.app.ui.catalog.cardCornerRadius(preset)

internal fun posterWidth(preset: String): androidx.compose.ui.unit.Dp = when (preset) {
    "xsmall" -> 64.dp
    "small" -> 78.dp
    "medium" -> 94.dp
    "large" -> 112.dp
    "xlarge" -> 132.dp
    else -> 94.dp
}

internal fun posterSpacing(preset: String): androidx.compose.ui.unit.Dp = com.fluxa.app.ui.catalog.cardRowSpacing(preset)

@Composable
internal fun SettingsPosterPreview(model: SettingsAppearanceHomeUiModel) {
    val shape = RoundedCornerShape(posterCornerRadius(model.cardCornerPreset))
    val width = posterWidth(model.posterWidthPreset)
    val aspectRatio = if (model.posterLandscapeMode) 16f / 9f else 2f / 3f
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 4.dp, bottom = 4.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .horizontalScroll(rememberScrollState())
            .padding(16.dp),
        horizontalArrangement = Arrangement.spacedBy(posterSpacing(model.interfaceDensity))
    ) {
        repeat(3) {
            Column(modifier = Modifier.width(width)) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(aspectRatio)
                        .clip(shape)
                        .background(Color.White.copy(alpha = 0.1f))
                        .border(1.dp, Color.White.copy(alpha = 0.08f), shape),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = Icons.Filled.Movie,
                        contentDescription = null,
                        tint = Color.White.copy(alpha = 0.25f),
                        modifier = Modifier.size(width / 3)
                    )
                }
                if (!model.posterHideTitles) {
                    Box(
                        modifier = Modifier
                            .padding(top = 6.dp)
                            .fillMaxWidth(0.75f)
                            .height(8.dp)
                            .clip(RoundedCornerShape(4.dp))
                            .background(Color.White.copy(alpha = 0.18f))
                    )
                }
            }
        }
    }
}

@Composable
internal fun SettingsAppearanceDetailContent(model: SettingsAppearanceDetailUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val styleOptions = listOf(
        SettingsChoiceOption("cinematic", AppStrings.t(lang, "settings.detail_style_cinematic")),
        SettingsChoiceOption("classic", AppStrings.t(lang, "settings.detail_style_classic")),
        SettingsChoiceOption("compact", AppStrings.t(lang, "settings.detail_style_compact"))
    )
    val seasonSelectorOptions = listOf(
        SettingsChoiceOption("dropdown", AppStrings.t(lang, "settings.season_selector_dropdown")),
        SettingsChoiceOption("tabs", AppStrings.t(lang, "settings.season_selector_tabs")),
        SettingsChoiceOption("posters", AppStrings.t(lang, "settings.season_selector_posters"))
    )
    val episodeLayoutOptions = listOf(
        SettingsChoiceOption("carousel", AppStrings.t(lang, "settings.episode_layout_horizontal")),
        SettingsChoiceOption("two_column", AppStrings.t(lang, "settings.episode_layout_grid")),
        SettingsChoiceOption("list", AppStrings.t(lang, "settings.episode_layout_list"))
    )

    SettingsSectionHeader(AppStrings.t(lang, "settings.detail_screen"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.detail_screen_style"), model.detailScreenStyle, styleOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailScreenStyle = it)))
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.detail_prefer_clearlogo"),
            description = AppStrings.t(lang, "settings.detail_prefer_clearlogo_desc"),
            value = model.detailPreferClearlogo
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailPreferClearlogo = it))) }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.detail_collapsing_hero"),
            description = AppStrings.t(lang, "settings.detail_collapsing_hero_desc"),
            value = model.detailCollapsingHero
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailCollapsingHero = it))) }
    }

    SettingsEpisodeLayoutPreview(model)
    SettingsSectionHeader(AppStrings.t(lang, "settings.episodes"))
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.detail_episode_descriptions"),
            value = model.detailShowEpisodeDescriptions
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailShowEpisodeDescriptions = it))) }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.blur_unwatched_episodes"),
            value = model.blurUnwatchedEpisodes
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(blurUnwatchedEpisodes = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.season_selector"), model.detailSeasonSelectorMode, seasonSelectorOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailSeasonSelectorMode = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.episode_cards_layout"), model.episodeCardsLayout, episodeLayoutOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(episodeCardsLayout = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.detail_sections"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.detail_show_cast"), value = model.detailShowCast) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailShowCast = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.detail_show_recommendations"), value = model.detailShowRecommendations) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailShowRecommendations = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_banner"))
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.trailer_on_detail_hero"),
            description = AppStrings.t(lang, "settings.trailer_on_detail_hero_desc"),
            value = model.trailerOnDetailHeroEnabled
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(trailerOnDetailHeroEnabled = it))) }
        if (model.trailerOnDetailHeroEnabled) {
            SettingsStepperRow(AppStrings.t(lang, "settings.trailer_on_detail_hero_delay"), model.trailerOnDetailHeroDelaySeconds, min = 0, max = 15, formatValue = { "${it}s" }) {
                onAction(SettingsAction.AppearanceDetailChanged(model.copy(trailerOnDetailHeroDelaySeconds = it)))
            }
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.season_posters_on_hero"),
            description = AppStrings.t(lang, "settings.detail_season_posters_on_hero_desc"),
            value = model.detailSeasonPostersOnHero
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailSeasonPostersOnHero = it))) }
    }
}

@Composable
internal fun SettingsEpisodeLayoutPreview(model: SettingsAppearanceDetailUiModel) {
    val thumbAlpha = if (model.blurUnwatchedEpisodes) 0.35f else 0.85f
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 4.dp, bottom = 4.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(16.dp)
    ) {
        if (model.episodeCardsLayout == "two_column") {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                repeat(2) {
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                        repeat(2) {
                            Column(modifier = Modifier.weight(1f)) {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .aspectRatio(16f / 9f)
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(Color.White.copy(alpha = thumbAlpha * 0.14f)),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White.copy(alpha = thumbAlpha), modifier = Modifier.size(18.dp))
                                }
                                Box(
                                    modifier = Modifier
                                        .padding(top = 5.dp)
                                        .fillMaxWidth(0.72f)
                                        .height(6.dp)
                                        .clip(RoundedCornerShape(4.dp))
                                        .background(Color.White.copy(alpha = 0.18f))
                                )
                            }
                        }
                    }
                }
            }
        } else if (model.episodeCardsLayout == "carousel") {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                repeat(3) {
                    Column(modifier = Modifier.width(96.dp)) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(16f / 9f)
                                .clip(RoundedCornerShape(8.dp))
                                .background(Color.White.copy(alpha = thumbAlpha * 0.14f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White.copy(alpha = thumbAlpha), modifier = Modifier.size(20.dp))
                        }
                        Box(
                            modifier = Modifier
                                .padding(top = 6.dp)
                                .fillMaxWidth(0.7f)
                                .height(7.dp)
                                .clip(RoundedCornerShape(4.dp))
                                .background(Color.White.copy(alpha = 0.18f))
                        )
                    }
                }
            }
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                repeat(2) {
                    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
                        Box(
                            modifier = Modifier
                                .width(84.dp)
                                .aspectRatio(16f / 9f)
                                .clip(RoundedCornerShape(8.dp))
                                .background(Color.White.copy(alpha = thumbAlpha * 0.14f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White.copy(alpha = thumbAlpha), modifier = Modifier.size(18.dp))
                        }
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth(0.6f)
                                    .height(8.dp)
                                    .clip(RoundedCornerShape(4.dp))
                                    .background(Color.White.copy(alpha = 0.2f))
                            )
                            Box(
                                modifier = Modifier
                                    .padding(top = 6.dp)
                                    .fillMaxWidth(0.9f)
                                    .height(6.dp)
                                    .clip(RoundedCornerShape(3.dp))
                                    .background(Color.White.copy(alpha = 0.1f))
                            )
                        }
                    }
                }
            }
        }
    }
}
