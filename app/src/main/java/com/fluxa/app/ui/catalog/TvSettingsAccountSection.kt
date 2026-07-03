@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.annotation.DrawableRes
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
            iconRes = com.fluxa.app.R.drawable.ic_trakt,
            value = if (!profile.traktAccessToken.isNullOrBlank()) {
                AppStrings.t(lang, "auto.connected")
            } else {
                AppStrings.t(lang, "auto.not_connected")
            },
            onClick = onConnectTrakt
        )
        SettingsConnectionTile(
            title = AppStrings.t(lang, "brand.myanimelist"),
            iconRes = com.fluxa.app.R.drawable.ic_myanimelist,
            value = if (!profile.malAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.connected") else AppStrings.t(lang, "auto.not_connected"),
            onClick = onConnectMal
        )
        SettingsConnectionTile(
            title = AppStrings.t(lang, "brand.simkl"),
            iconRes = com.fluxa.app.R.drawable.ic_simkl,
            value = if (!profile.simklAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.connected") else AppStrings.t(lang, "auto.not_connected"),
            onClick = onConnectSimkl
        )
        if (!profile.traktAccessToken.isNullOrBlank() || !profile.malAccessToken.isNullOrBlank() || !profile.simklAccessToken.isNullOrBlank()) {
            SettingsActionTile(
                title = AppStrings.t(lang, "auto.disconnect"),
                subtitle = AppStrings.t(lang, "integration.connected_accounts"),
                icon = FluxaIcons.Logout,
                accent = FluxaColors.errorRed,
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
