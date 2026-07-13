package com.fluxa.app.shared.feature.profile

import kotlinx.coroutines.flow.Flow

data class ProfileUiModel(
    val id: String,
    val name: String,
    val avatarUrl: String?,
    val language: String,
    val accentColorArgb: Long
)

data class ProfileUiState(
    val activeProfile: ProfileUiModel? = null,
    val profiles: List<ProfileUiModel> = emptyList(),
    val isLoading: Boolean = false
)

data class SettingsUiState(
    val language: String = "en",
    val cardLayout: String = "vertical",
    val autoPlayNextEpisode: Boolean = true,
    val subtitlesEnabled: Boolean = false,
    val preferredAudioLanguage: String = "none",
    val preferredSubtitleLanguage: String = "none"
)

interface ProfileDataSource {
    fun observeProfiles(): Flow<ProfileUiState>
    fun observeSettings(profileId: String): Flow<SettingsUiState>
    suspend fun selectProfile(id: String)
    suspend fun updateSettings(profileId: String, settings: SettingsUiState)
}
