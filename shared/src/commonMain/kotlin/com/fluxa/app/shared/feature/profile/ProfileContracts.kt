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
    val pinError: Boolean = false,
    val pickerBackgroundUrl: String? = null,
    val avatarPacks: List<ProfileAvatarPackUiModel> = emptyList(),
    val avatarPackDiscoveryInProgress: Boolean = false,
    val avatarPackDiscoveryError: Boolean = false
)

data class ProfileAvatarPackUiModel(
    val id: String,
    val repositoryUrl: String,
    val title: String,
    val avatars: List<ProfileAvatarUiModel>
)

data class ProfileAvatarUiModel(
    val name: String,
    val url: String
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
    data object PickerSettingsRequested : ProfileAction
    data object PickerSettingsClosed : ProfileAction
    data class BackgroundUrlChanged(val url: String?) : ProfileAction
    data class AddAvatarPackRequested(val repositoryUrl: String) : ProfileAction
    data class RemoveAvatarPackRequested(val packId: String) : ProfileAction
    data class RefreshAvatarPackRequested(val repositoryUrl: String) : ProfileAction
}

sealed interface ProfileEditTarget {
    data object New : ProfileEditTarget
    data class Existing(val id: String) : ProfileEditTarget
}

interface ProfileDataSource {
    fun observeProfiles(): Flow<ProfileUiState>
    suspend fun selectProfile(id: String)
    suspend fun attemptPin(profileId: String, pin: String)
    suspend fun confirmBiometricUnlock(profileId: String)
    suspend fun cancelPinUnlock()
    suspend fun deleteProfile(id: String)
    suspend fun saveProfile(edit: ProfileEditUiModel): String
    suspend fun setPickerBackground(url: String?)
    suspend fun addAvatarPack(repositoryUrl: String)
    suspend fun removeAvatarPack(packId: String)
    suspend fun refreshAvatarPack(repositoryUrl: String)
    suspend fun refreshAllAvatarPacks()
}
