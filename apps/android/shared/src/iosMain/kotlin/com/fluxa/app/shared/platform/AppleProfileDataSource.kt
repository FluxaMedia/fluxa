package com.fluxa.app.shared.platform

import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.PinHasher
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfilePersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import platform.Foundation.NSUserDefaults

class AppleProfileDataSource(
    onActiveProfileChanged: (String) -> Unit = {}
) : ProfileDataSource by SharedProfileDataSource(AppleProfileStore(onActiveProfileChanged))

private class AppleProfileStore(
    private val onActiveProfileChanged: (String) -> Unit
) : ProfilePersistence {
    private val defaults = NSUserDefaults.standardUserDefaults
    private val state = MutableStateFlow(snapshot())

    init {
        onActiveProfileChanged(defaults.stringForKey(ActiveProfileKey) ?: ProfileId)
    }

    override fun observe(): Flow<ProfileStoreSnapshot> = state.asStateFlow()

    override suspend fun pinHash(profileId: String): String? {
        val stored = defaults.stringForKey(PinHashKey) ?: defaults.stringForKey(LegacyPinKey) ?: return null
        return stored.takeIf { it.length == 64 } ?: PinHasher.hash(stored)
    }

    override suspend fun activate(profileId: String) {
        defaults.setObject(profileId, ActiveProfileKey)
        onActiveProfileChanged(profileId)
        state.value = snapshot()
    }

    override suspend fun delete(profileId: String) {
        listOf(NameKey, AvatarUrlKey, PinHashKey, LegacyPinKey, BiometricEnabledKey, ActiveProfileKey).forEach(defaults::removeObjectForKey)
        state.value = snapshot()
    }

    override suspend fun save(edit: ProfileEditUiModel, pinHash: String?): String {
        defaults.setObject(edit.name, NameKey)
        edit.avatarUrl?.let { defaults.setObject(it, AvatarUrlKey) } ?: defaults.removeObjectForKey(AvatarUrlKey)
        pinHash?.let { defaults.setObject(it, PinHashKey) } ?: defaults.removeObjectForKey(PinHashKey)
        defaults.removeObjectForKey(LegacyPinKey)
        defaults.setBool(edit.biometricEnabled, BiometricEnabledKey)
        defaults.setObject(ProfileId, ActiveProfileKey)
        onActiveProfileChanged(ProfileId)
        state.value = snapshot()
        return ProfileId
    }

    private fun snapshot(): ProfileStoreSnapshot {
        val profile = readProfile()
        val active = defaults.stringForKey(ActiveProfileKey)?.takeIf { it == ProfileId }?.let { profile }
        return ProfileStoreSnapshot(activeProfile = active ?: profile, profiles = listOf(profile))
    }

    private fun readProfile(): ProfileUiModel {
        val language = defaults.stringForKey(LanguageKey) ?: "en"
        return ProfileUiModel(
            id = ProfileId,
            name = defaults.stringForKey(NameKey) ?: AppStrings.t(language, "auto.default"),
            avatarUrl = defaults.stringForKey(AvatarUrlKey),
            language = language,
            accentColorArgb = 0xff6750a4L,
            hasPin = !defaults.stringForKey(PinHashKey).isNullOrBlank() || !defaults.stringForKey(LegacyPinKey).isNullOrBlank(),
            biometricEnabled = defaults.objectForKey(BiometricEnabledKey)?.let { defaults.boolForKey(BiometricEnabledKey) } ?: false
        )
    }

    private companion object {
        const val ProfileId = "apple-default"
        const val LanguageKey = "fluxa.apple.language"
        const val NameKey = "fluxa.apple.profile.name"
        const val AvatarUrlKey = "fluxa.apple.profile.avatarUrl"
        const val PinHashKey = "fluxa.apple.profile.pinHash"
        const val LegacyPinKey = "fluxa.apple.profile.pin"
        const val BiometricEnabledKey = "fluxa.apple.profile.biometricEnabled"
        const val ActiveProfileKey = "fluxa.apple.profile.activeId"
    }
}
