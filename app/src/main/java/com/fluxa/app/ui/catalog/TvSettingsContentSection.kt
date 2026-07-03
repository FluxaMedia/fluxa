@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun ContentSettings(
    profile: UserProfile,
    lang: String,
    categories: List<HomeCategory>,
    userAddons: List<AddonDescriptor>,
    cs3FeedOptions: List<MetadataFeedOption>,
    onUpdateProfile: (UserProfile) -> Unit
) {
    val metadataFeedOptions = buildMetadataFeedOptions(userAddons, lang) + cs3FeedOptions
    val heroFeedOptions = orderedMetadataFeeds(metadataFeedOptions, profile.heroFeedOrder)
    val homeFeedOptions = orderedMetadataFeeds(metadataFeedOptions, profile.homeFeedOrder)
    val metadataFeedKeys = metadataFeedOptions.map { it.key }
    val heroSelectedKeys = effectiveMetadataFeedSelection(profile.heroFeedToggles, metadataFeedKeys) ?: metadataFeedKeys.take(2)
    val homeSelectedKeys = effectiveMetadataFeedSelection(profile.homeFeedToggles, metadataFeedKeys)
    val homeVisibleFeedOptions = homeFeedOptions.filter { isMetadataFeedEnabled(homeSelectedKeys, it.key) }
    val topTenSelectedKeys = profile.safeTopTenFeedToggles

    SettingsSection(AppStrings.t(lang, "settings.hero_catalogs"), "") {
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.show_hero_section"),
            subtitle = AppStrings.t(lang, "settings.show_hero_section_desc"),
            checked = profile.safeShowHeroSection,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(showHeroSection = it)) }
        )
        heroFeedOptions.forEachIndexed { index, option ->
            SettingsOrderedToggleTile(
                title = option.label,
                checked = heroSelectedKeys.contains(option.key),
                onToggle = {
                    onUpdateProfile(profile.sanitizedUpdate(heroFeedToggles = toggleMetadataFeed(heroSelectedKeys, metadataFeedKeys, option.key, maxEnabled = 2)))
                },
                canMoveUp = index > 0,
                canMoveDown = index < heroFeedOptions.lastIndex,
                onMoveUp = { onUpdateProfile(profile.sanitizedUpdate(heroFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.heroFeedOrder, option.key, -1))) },
                onMoveDown = { onUpdateProfile(profile.sanitizedUpdate(heroFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.heroFeedOrder, option.key, 1))) }
            )
        }
    }
    SettingsSection(AppStrings.t(lang, "settings.home_catalogs"), AppStrings.t(lang, "settings.home_catalogs_desc")) {
        homeFeedOptions.forEachIndexed { index, option ->
            SettingsOrderedToggleTile(
                title = option.label,
                checked = isMetadataFeedEnabled(homeSelectedKeys, option.key),
                onToggle = {
                    onUpdateProfile(profile.sanitizedUpdate(
                        homeFeedToggles = toggleMetadataFeed(homeSelectedKeys, metadataFeedKeys, option.key),
                        cs3FeedsConfigured = if (cs3FeedOptions.isNotEmpty()) true else profile.cs3FeedsConfigured
                    ))
                },
                canMoveUp = index > 0,
                canMoveDown = index < homeFeedOptions.lastIndex,
                onMoveUp = { onUpdateProfile(profile.sanitizedUpdate(homeFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.homeFeedOrder, option.key, -1))) },
                onMoveDown = { onUpdateProfile(profile.sanitizedUpdate(homeFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.homeFeedOrder, option.key, 1))) }
            )
        }
    }
    SettingsSection(AppStrings.t(lang, "settings.top_10_catalogs"), AppStrings.t(lang, "settings.top_10_catalogs_desc")) {
        homeVisibleFeedOptions.forEach { option ->
            SettingsToggleTile(
                title = option.label,
                subtitle = "",
                checked = topTenSelectedKeys.contains(option.key),
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(topTenFeedToggles = toggleTopTenFeed(topTenSelectedKeys, option.key))) }
            )
        }
    }
}

@Composable
internal fun AdvancedSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "settings.advanced_settings"), AppStrings.t(lang, "settings.advanced_settings_desc")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.buffer_cache"),
            subtitle = AppStrings.t(lang, "settings.buffer_cache_desc"),
            options = listOf(
                ChoiceOption("100", "100 MB"),
                ChoiceOption("500", "500 MB"),
                ChoiceOption("1000", "1 GB"),
                ChoiceOption("2000", "2 GB")
            ),
            selected = profile.safePlayerBufferCacheMb.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerBufferCacheMb = it.toInt())) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.forward_buffer"),
            subtitle = AppStrings.t(lang, "settings.forward_buffer_desc"),
            options = bufferSecondOptions(lang),
            selected = profile.safePlayerForwardBufferSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerForwardBufferSeconds = it.toInt())) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.back_buffer"),
            subtitle = AppStrings.t(lang, "settings.back_buffer_desc"),
            options = bufferSecondOptions(lang, includeZero = true),
            selected = profile.safePlayerBackBufferSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerBackBufferSeconds = it.toInt())) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.audio_decoder_mode"),
            subtitle = AppStrings.t(lang, "settings.audio_decoder_mode_desc"),
            options = audioDecoderModeOptions(lang),
            selected = profile.safeAudioDecoderMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(audioDecoderMode = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.tunneled_playback"),
            subtitle = AppStrings.t(lang, "settings.tunneled_playback_desc"),
            checked = profile.safeTunneledPlayback,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(tunneledPlayback = it)) }
        )
    }
}
