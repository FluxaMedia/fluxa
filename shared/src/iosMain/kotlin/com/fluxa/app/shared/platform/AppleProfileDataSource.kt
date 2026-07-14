package com.fluxa.app.shared.platform

import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import platform.Foundation.NSUserDefaults

class AppleProfileDataSource : ProfileDataSource {
    private val defaults = NSUserDefaults.standardUserDefaults
    private val profiles = MutableStateFlow(ProfileUiState(activeProfile = readProfile(), profiles = listOf(readProfile())))

    override fun observeProfiles(): Flow<ProfileUiState> = profiles.asStateFlow()

    override suspend fun selectProfile(id: String) = Unit

    override suspend fun attemptPin(profileId: String, pin: String) {
        val pendingProfile = profiles.value.pendingPinProfile ?: return
        val storedPin = defaults.stringForKey(PinKey)
        if (storedPin.isNullOrBlank() || storedPin == pin) {
            profiles.value = profiles.value.copy(pendingPinProfile = null, pinError = false)
        } else {
            profiles.value = profiles.value.copy(pinError = true)
        }
    }

    override suspend fun confirmBiometricUnlock(profileId: String) {
        profiles.value = profiles.value.copy(pendingPinProfile = null, pinError = false)
    }

    override suspend fun cancelPinUnlock() {
        profiles.value = profiles.value.copy(pendingPinProfile = null, pinError = false)
    }

    override suspend fun deleteProfile(id: String) = Unit

    override suspend fun saveProfile(edit: ProfileEditUiModel): String {
        defaults.setObject(edit.name, NameKey)
        edit.avatarUrl?.let { defaults.setObject(it, AvatarUrlKey) } ?: defaults.removeObjectForKey(AvatarUrlKey)
        if (!edit.keepExistingPin) {
            edit.newPin?.let { defaults.setObject(it, PinKey) } ?: defaults.removeObjectForKey(PinKey)
        }
        defaults.setBool(edit.biometricEnabled, BiometricEnabledKey)
        val updated = readProfile()
        profiles.value = profiles.value.copy(activeProfile = updated, profiles = listOf(updated))
        return updated.id
    }

    private fun readProfile(): ProfileUiModel {
        val language = defaults.stringForKey(LanguageKey) ?: "en"
        return ProfileUiModel(
            id = ProfileId,
            name = defaults.stringForKey(NameKey) ?: AppStrings.t(language, "auto.default"),
            avatarUrl = defaults.stringForKey(AvatarUrlKey),
            language = language,
            accentColorArgb = 0xff6750a4L,
            hasPin = !defaults.stringForKey(PinKey).isNullOrBlank(),
            biometricEnabled = defaults.objectForKey(BiometricEnabledKey)?.let { defaults.boolForKey(BiometricEnabledKey) } ?: false
        )
    }

    private companion object {
        const val ProfileId = "apple-default"
        const val LanguageKey = "fluxa.apple.language"
        const val NameKey = "fluxa.apple.profile.name"
        const val AvatarUrlKey = "fluxa.apple.profile.avatarUrl"
        const val PinKey = "fluxa.apple.profile.pin"
        const val BiometricEnabledKey = "fluxa.apple.profile.biometricEnabled"
    }
}
