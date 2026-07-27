package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProfilePickerSettingsStore
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAccentColorArgb
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.repository.ProfileAvatarPackRepository
import com.fluxa.app.shared.feature.profile.ProfileAvatarPackUiModel
import com.fluxa.app.shared.feature.profile.ProfileAvatarUiModel
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfilePersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import java.util.UUID
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

class AndroidProfileDataSource(
    profileManager: ProfileManager,
    pickerSettingsStore: ProfilePickerSettingsStore
) : ProfileDataSource by SharedProfileDataSource(AndroidProfileStore(profileManager, pickerSettingsStore))

private class AndroidProfileStore(
    private val profileManager: ProfileManager,
    private val pickerSettingsStore: ProfilePickerSettingsStore
) : ProfilePersistence {
    private val avatarPackRepository = ProfileAvatarPackRepository()

    override fun observe(): Flow<ProfileStoreSnapshot> = callbackFlow {
        fun emit() {
            val profiles = profileManager.getProfiles()
            val activeId = profileManager.getLastActiveProfileId()
            val pickerSettings = pickerSettingsStore.get()
            trySend(
                ProfileStoreSnapshot(
                    activeProfile = profiles.firstOrNull { it.id == activeId }?.toUiModel(),
                    profiles = profiles.map(UserProfile::toUiModel),
                    pickerBackgroundUrl = pickerSettings.backgroundUrl,
                    avatarPacks = pickerSettings.avatarPacks.map { pack ->
                        ProfileAvatarPackUiModel(
                            id = pack.id,
                            repositoryUrl = pack.repositoryUrl,
                            title = pack.title,
                            avatars = pack.avatars.map { ProfileAvatarUiModel(name = it.name, url = it.url) }
                        )
                    }
                )
            )
        }
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> emit() }
        profileManager.registerOnChangeListener(listener)
        pickerSettingsStore.registerOnChangeListener(listener)
        emit()
        awaitClose {
            profileManager.unregisterOnChangeListener(listener)
            pickerSettingsStore.unregisterOnChangeListener(listener)
        }
    }

    override suspend fun setPickerBackground(url: String?) {
        pickerSettingsStore.save(pickerSettingsStore.get().copy(backgroundUrl = url))
    }

    override suspend fun addAvatarPack(repositoryUrl: String): Result<Unit> {
        return avatarPackRepository.discover(repositoryUrl).map { discovered ->
            val current = pickerSettingsStore.get()
            val merged = (current.avatarPacks.filterNot { existing -> discovered.any { it.id == existing.id } } + discovered)
            pickerSettingsStore.save(current.copy(avatarPacks = merged))
        }
    }

    override suspend fun removeAvatarPack(packId: String) {
        val current = pickerSettingsStore.get()
        pickerSettingsStore.save(current.copy(avatarPacks = current.avatarPacks.filterNot { it.id == packId }))
    }

    override suspend fun refreshAvatarPack(repositoryUrl: String): Result<Unit> {
        return avatarPackRepository.discover(repositoryUrl).map { refreshed ->
            val current = pickerSettingsStore.get()
            val kept = current.avatarPacks.filterNot { it.repositoryUrl == repositoryUrl }
            pickerSettingsStore.save(current.copy(avatarPacks = kept + refreshed))
        }
    }

    override suspend fun pinHash(profileId: String): String? =
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.pinHash

    override suspend fun activate(profileId: String) {
        profileManager.setLastActiveProfile(profileManager.getProfiles().firstOrNull { it.id == profileId })
    }

    override suspend fun delete(profileId: String) {
        profileManager.deleteProfileById(profileId)
    }

    override suspend fun save(edit: ProfileEditUiModel, pinHash: String?): String {
        val existing = edit.id?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
        val profile = existing?.copy(
            profileName = edit.name,
            avatarUrl = edit.avatarUrl,
            pinHash = pinHash,
            biometricEnabled = edit.biometricEnabled
        ) ?: UserProfile(
            id = UUID.randomUUID().toString(),
            email = edit.name,
            profileName = edit.name,
            authKey = "",
            language = "en",
            avatarUrl = edit.avatarUrl,
            pinHash = pinHash,
            biometricEnabled = edit.biometricEnabled,
            localAddons = listOf("https://v3-cinemeta.strem.io/manifest.json")
        )
        profileManager.saveProfile(profile)
        return profile.id
    }
}

private fun UserProfile.toUiModel(): ProfileUiModel = ProfileUiModel(
    id = id,
    name = profileName?.takeIf { it.isNotBlank() } ?: email,
    avatarUrl = avatarUrl,
    language = safeLanguage,
    accentColorArgb = safeAccentColorArgb.toLong() and 0xffffffffL,
    hasPin = !pinHash.isNullOrBlank(),
    biometricEnabled = biometricEnabled == true
)
