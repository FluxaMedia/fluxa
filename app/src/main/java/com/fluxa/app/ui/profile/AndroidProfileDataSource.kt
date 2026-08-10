package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProfilePickerSettingsStore
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.repository.ProfileAvatarPackRepository
import com.fluxa.app.common.PinHasher
import com.fluxa.app.shared.feature.profile.ProfileAvatarPackUiModel
import com.fluxa.app.shared.feature.profile.ProfileAvatarUiModel
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.JvmPbkdf2PinHasher
import com.fluxa.app.shared.feature.profile.ProfileBase64Codec
import com.fluxa.app.shared.feature.profile.ProfileManagerPersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import com.fluxa.app.shared.feature.profile.toProfileUiModel
import android.util.Base64
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AndroidProfileDataSource(
    profileManager: ProfileManager,
    pickerSettingsStore: ProfilePickerSettingsStore
) : ProfileDataSource by SharedProfileDataSource(AndroidProfileStore(profileManager, pickerSettingsStore))

private class AndroidProfileStore(
    profileManager: ProfileManager,
    private val pickerSettingsStore: ProfilePickerSettingsStore,
) : ProfileManagerPersistence(
    profileManager = profileManager,
    initializeNewProfile = { profile ->
        profile.copy(localAddons = listOf("https://v3-cinemeta.strem.io/manifest.json"))
    },
) {
    private val avatarPackRepository = ProfileAvatarPackRepository()
    private val state = MutableStateFlow(snapshot())

    init {
        profileManager.addChangeListener { state.value = snapshot() }
        pickerSettingsStore.registerOnChangeListener(
            SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> state.value = snapshot() }
        )
    }

    private fun snapshot(): ProfileStoreSnapshot {
        val profiles = profileManager.getProfiles()
        val activeId = profileManager.getLastActiveProfileId()
        val pickerSettings = pickerSettingsStore.get()
        return ProfileStoreSnapshot(
            activeProfile = profiles.firstOrNull { it.id == activeId }?.toProfileUiModel(),
            profiles = profiles.map(UserProfile::toProfileUiModel),
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
    }

    override fun observe(): Flow<ProfileStoreSnapshot> = state.asStateFlow()

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

    private val pinHasher = JvmPbkdf2PinHasher(AndroidBase64Codec)

    override suspend fun createPinHash(pin: String): String = pinHasher.hash(pin)

    override suspend fun verifyPin(profileId: String, pin: String, storedHash: String): Boolean {
        if (pinHasher.verify(pin, storedHash)) return true
        if (storedHash.length != 64 || PinHasher.hash(pin) != storedHash) return false
        profileManager.updateProfile(profileId) { it.copy(pinHash = pinHasher.hash(pin)) }
        return true
    }
}

private object AndroidBase64Codec : ProfileBase64Codec {
    override fun encode(bytes: ByteArray): String = Base64.encodeToString(bytes, Base64.NO_WRAP)
    override fun decode(value: String): ByteArray? =
        runCatching { Base64.decode(value, Base64.NO_WRAP) }.getOrNull()
}
