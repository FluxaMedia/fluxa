package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProfilePickerSettingsStore
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.repository.ProfileAvatarPackRepository
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.NuvioCoreBridge
import com.fluxa.app.common.PinHasher
import com.fluxa.app.shared.feature.profile.ProfileAvatarPackUiModel
import com.fluxa.app.shared.feature.profile.ProfileAvatarUiModel
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.JvmPbkdf2PinHasher
import com.fluxa.app.shared.feature.profile.ProfileBase64Codec
import com.fluxa.app.shared.feature.profile.ProfileManagerPersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import com.fluxa.app.shared.feature.profile.toProfileUiModel
import android.util.Base64
import com.google.gson.JsonParser
import java.util.UUID
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AndroidProfileDataSource(
    profileManager: ProfileManager,
    pickerSettingsStore: ProfilePickerSettingsStore,
    nuvioImportCoordinator: NuvioAccountImportCoordinator,
) : ProfileDataSource by SharedProfileDataSource(AndroidProfileStore(profileManager, pickerSettingsStore, nuvioImportCoordinator))

private class AndroidProfileStore(
    profileManager: ProfileManager,
    private val pickerSettingsStore: ProfilePickerSettingsStore,
    private val nuvioImportCoordinator: NuvioAccountImportCoordinator,
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

    override suspend fun hasPin(profileId: String): Boolean =
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { profile ->
            !profile.pinHash.isNullOrBlank() || profile.nuvioPinEnabled
        } == true

    override suspend fun verifyPin(profileId: String, pin: String, storedHash: String): Boolean {
        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId }
        if (profile?.nuvioPinEnabled == true) {
            val cache = profileManager.getNuvioPinCache(profileId)?.let { JsonParser.parseString(it) }
            val cached = NuvioCoreBridge.verifyCachedPin(
                profileIndex = profile.nuvioProfileIndex ?: 1,
                pin = pin,
                pinEnabled = true,
                profileUpdatedAt = profile.nuvioProfileUpdatedAt,
                cache = cache,
            )
            if (cached.get("unlocked")?.asBoolean == true) return true
            if (cached.get("reason")?.asString == "profile_changed") profileManager.clearNuvioPinCache(profileId)

            val remote = runCatching { nuvioImportCoordinator.verifyNuvioProfilePin(profile, pin) }.getOrNull()
            if (remote?.unlocked == true) {
                val payload = NuvioCoreBridge.pinCachePayload(
                    profileIndex = profile.nuvioProfileIndex ?: 1,
                    salt = UUID.randomUUID().toString(),
                    pin = pin,
                    profileUpdatedAt = profile.nuvioProfileUpdatedAt,
                )
                profileManager.saveNuvioPinCache(profileId, payload.toString())
                return true
            }
            return false
        }
        if (pinHasher.verify(pin, storedHash)) return true
        if (storedHash.length != 64 || PinHasher.hash(pin) != storedHash) return false
        profileManager.updateProfile(profileId) { it.copy(pinHash = pinHasher.hash(pin)) }
        return true
    }

    override suspend fun syncRemotePin(edit: ProfileEditUiModel): Result<Unit> = runCatching {
        val profile = edit.id?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
            ?: return@runCatching
        if (profile.nuvioAccessToken.isNullOrBlank() || profile.nuvioProfileIndex == null) return@runCatching

        val newPin = edit.newPin
        when {
            newPin != null -> {
                nuvioImportCoordinator.setNuvioProfilePin(profile, newPin, edit.currentPin)
                profileManager.updateProfile(profile.id) { it.copy(nuvioPinEnabled = true) }
            }
            !edit.keepExistingPin && profile.nuvioPinEnabled -> {
                nuvioImportCoordinator.clearNuvioProfilePin(profile, edit.currentPin)
                profileManager.updateProfile(profile.id) { it.copy(nuvioPinEnabled = false, nuvioPinLockedUntil = null) }
                profileManager.clearNuvioPinCache(profile.id)
            }
        }
    }
}

private object AndroidBase64Codec : ProfileBase64Codec {
    override fun encode(bytes: ByteArray): String = Base64.encodeToString(bytes, Base64.NO_WRAP)
    override fun decode(value: String): ByteArray? =
        runCatching { Base64.decode(value, Base64.NO_WRAP) }.getOrNull()
}
