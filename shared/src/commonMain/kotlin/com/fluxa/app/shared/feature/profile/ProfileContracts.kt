package com.fluxa.app.shared.feature.profile

import kotlinx.coroutines.flow.Flow

data class ProfileUiModel(
    val id: String,
    val name: String,
    val avatarUrl: String?,
    val language: String,
    val accentColorArgb: Long,
    val hasPin: Boolean = false,
    val biometricEnabled: Boolean = false
)

data class ProfileUiState(
    val activeProfile: ProfileUiModel? = null,
    val profiles: List<ProfileUiModel> = emptyList(),
    val isLoading: Boolean = false,
    val pendingPinProfile: ProfileUiModel? = null,
    val pinError: Boolean = false
)

data class SettingsUiState(
    val language: String = "en",
    val cardLayout: String = "vertical",
    val autoPlayNextEpisode: Boolean = true,
    val subtitlesEnabled: Boolean = false,
    val preferredAudioLanguage: String = "none",
    val preferredSubtitleLanguage: String = "none"
)

data class ProfileEditUiModel(
    val id: String? = null,
    val name: String,
    val avatarUrl: String?,
    val newPin: String? = null,
    val keepExistingPin: Boolean = false,
    val biometricEnabled: Boolean
)

sealed interface ProfileAction {
    data class Selected(val profile: ProfileUiModel) : ProfileAction
    data object AddRequested : ProfileAction
    data class EditRequested(val profile: ProfileUiModel) : ProfileAction
    data class PinAttempt(val profileId: String, val pin: String) : ProfileAction
    data object PinCancelled : ProfileAction
}

sealed interface ProfileEditTarget {
    data object New : ProfileEditTarget
    data class Existing(val id: String) : ProfileEditTarget
}

interface ProfileDataSource {
    fun observeProfiles(): Flow<ProfileUiState>
    fun observeSettings(profileId: String): Flow<SettingsUiState>
    suspend fun selectProfile(id: String)
    suspend fun updateSettings(profileId: String, settings: SettingsUiState)
    suspend fun attemptPin(profileId: String, pin: String)
    suspend fun confirmBiometricUnlock(profileId: String)
    suspend fun cancelPinUnlock()
    suspend fun deleteProfile(id: String)
    suspend fun saveProfile(edit: ProfileEditUiModel): String
}
