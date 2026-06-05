@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.foundation.Image
import androidx.compose.animation.animateColorAsState
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.R
import com.fluxa.app.data.local.UserProfile

@Composable
internal fun MobileTmdbApiSettings(
    profile: UserProfile,
    lang: String,
    onUpdateProfile: (UserProfile) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    val tmdbGreen = Color(0xFF90CEA1)
    val tmdbBlue = Color(0xFF01B4E4)
    val tmdbDark = Color(0xFF0D253F)
    val hasKey = profile.safeTmdbApiKey.isNotBlank()
    var isFocused by remember { mutableStateOf(false) }
    var showKey by remember { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(0.dp)) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 10.dp)
        ) {
            val borderColor by animateColorAsState(
                targetValue = when {
                    isFocused -> tmdbBlue
                    hasKey -> tmdbGreen.copy(alpha = 0.55f)
                    else -> colors.text.copy(alpha = 0.12f)
                },
                animationSpec = tween(200),
                label = "tmdbBorder"
            )
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(tmdbDark.copy(alpha = 0.45f))
                    .border(1.dp, borderColor, RoundedCornerShape(14.dp))
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.ic_tmdb),
                        contentDescription = null,
                        modifier = Modifier.size(22.dp)
                    )
                    Text(
                        text = AppStrings.t(lang, "settings.tmdb_api_key"),
                        color = colors.text.copy(alpha = 0.55f),
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                        letterSpacing = 0.6.sp
                    )
                    Spacer(Modifier.weight(1f))
                    if (hasKey) {
                        Box(
                            modifier = Modifier
                                .background(tmdbGreen.copy(alpha = 0.15f), RoundedCornerShape(20.dp))
                                .padding(horizontal = 8.dp, vertical = 3.dp)
                        ) {
                            Text(
                                text = if (settingsIsEnglish(lang)) "Active" else "Aktif",
                                color = tmdbGreen,
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold
                            )
                        }
                    }
                }
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    androidx.compose.foundation.text.BasicTextField(
                        value = profile.safeTmdbApiKey,
                        onValueChange = { onUpdateProfile(profile.copy(tmdbApiKey = it.trim())) },
                        modifier = Modifier
                            .weight(1f)
                            .onFocusChanged { isFocused = it.isFocused },
                        singleLine = true,
                        visualTransformation = if (showKey || profile.safeTmdbApiKey.isBlank()) {
                            VisualTransformation.None
                        } else {
                            PasswordVisualTransformation('•')
                        },
                        textStyle = TextStyle(
                            color = colors.text.copy(alpha = 0.92f),
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Medium
                        ),
                        cursorBrush = SolidColor(tmdbBlue),
                        decorationBox = { inner ->
                            if (profile.safeTmdbApiKey.isBlank()) {
                                Text(
                                    text = AppStrings.t(lang, "settings.tmdb_api_key_placeholder"),
                                    color = colors.text.copy(alpha = 0.28f),
                                    fontSize = 14.sp
                                )
                            }
                            inner()
                        }
                    )
                    if (hasKey) {
                        Icon(
                            imageVector = if (showKey) FluxaIcons.Visibility else FluxaIcons.VisibilityOff,
                            contentDescription = null,
                            tint = colors.text.copy(alpha = 0.38f),
                            modifier = Modifier
                                .size(18.dp)
                                .clickable { showKey = !showKey }
                        )
                    }
                }
                val lineWidth by animateFloatAsState(
                    targetValue = if (isFocused) 1f else 0f,
                    animationSpec = tween(250),
                    label = "tmdbLine"
                )
                if (lineWidth > 0f) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(lineWidth)
                            .height(1.5.dp)
                            .background(
                                Brush.horizontalGradient(listOf(tmdbGreen, tmdbBlue)),
                                RoundedCornerShape(1.dp)
                            )
                    )
                }
            }
        }
        if (profile.safeTmdbApiKey.isNotBlank()) {
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_cast_images"),
                checked = profile.safeTmdbCastImagesEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbCastImagesEnabled = !profile.safeTmdbCastImagesEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_similar_results"),
                checked = profile.safeTmdbSimilarResultsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbSimilarResultsEnabled = !profile.safeTmdbSimilarResultsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_trailers"),
                checked = profile.safeTmdbTrailersEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbTrailersEnabled = !profile.safeTmdbTrailersEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_recommendations"),
                checked = profile.safeTmdbRecommendationsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbRecommendationsEnabled = !profile.safeTmdbRecommendationsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_collection_info"),
                checked = profile.safeTmdbCollectionInfoEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbCollectionInfoEnabled = !profile.safeTmdbCollectionInfoEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_episode_images"),
                checked = profile.safeTmdbEpisodeImagesEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbEpisodeImagesEnabled = !profile.safeTmdbEpisodeImagesEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_logos_backdrops"),
                checked = profile.safeTmdbLogosBackdropsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbLogosBackdropsEnabled = !profile.safeTmdbLogosBackdropsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_ratings"),
                checked = profile.safeTmdbRatingsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbRatingsEnabled = !profile.safeTmdbRatingsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_basic_info"),
                checked = profile.safeTmdbBasicInfoEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbBasicInfoEnabled = !profile.safeTmdbBasicInfoEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_details"),
                checked = profile.safeTmdbDetailsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbDetailsEnabled = !profile.safeTmdbDetailsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_productions"),
                checked = profile.safeTmdbProductionsEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbProductionsEnabled = !profile.safeTmdbProductionsEnabled)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.tmdb_networks"),
                checked = profile.safeTmdbNetworksEnabled,
                onToggle = { onUpdateProfile(profile.copy(tmdbNetworksEnabled = !profile.safeTmdbNetworksEnabled)) }
            )
        }
    }
}
