package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProfilePickerSettingsStore
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAccentColorArgb
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.repository.ProfileAvatarPackRepository
import com.fluxa.app.common.PinHasher
import com.fluxa.app.shared.feature.profile.ProfileAvatarPackUiModel
import com.fluxa.app.shared.feature.profile.ProfileAvatarUiModel
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfilePersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import java.util.UUID
import java.security.SecureRandom
import java.security.MessageDigest
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec
import android.util.Base64
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AndroidProfileDataSource(
    profileManager: ProfileManager,
    pickerSettingsStore: ProfilePickerSettingsStore
) : ProfileDataSource by SharedProfileDataSource(AndroidProfileStore(profileManager, pickerSettingsStore))

private class AndroidProfileStore(
    private val profileManager: ProfileManager,
    private val pickerSettingsStore: ProfilePickerSettingsStore
) : ProfilePersistence {
    private val avatarPackRepository = ProfileAvatarPackRepository()
    private val state = MutableStateFlow(snapshot())

    init {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> state.value = snapshot() }
        profileManager.registerOnChangeListener(listener)
        pickerSettingsStore.registerOnChangeListener(listener)
    }

    private fun snapshot(): ProfileStoreSnapshot {
        val profiles = profileManager.getProfiles()
        val activeId = profileManager.getLastActiveProfileId()
        val pickerSettings = pickerSettingsStore.get()
        return ProfileStoreSnapshot(
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

    override suspend fun pinHash(profileId: String): String? =
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.pinHash

    override suspend fun createPinHash(pin: String): String = AndroidPinHasher.hash(pin)

    override suspend fun verifyPin(profileId: String, pin: String, storedHash: String): Boolean {
        if (AndroidPinHasher.verify(pin, storedHash)) return true
        if (storedHash.length != 64 || PinHasher.hash(pin) != storedHash) return false
        profileManager.updateProfile(profileId) { it.copy(pinHash = AndroidPinHasher.hash(pin)) }
        return true
    }

    override suspend fun canAttemptPin(profileId: String): Boolean = profileManager.canAttemptPin(profileId)

    override suspend fun recordPinFailure(profileId: String) {
        profileManager.recordPinFailure(profileId)
    }

    override suspend fun clearPinFailures(profileId: String) {
        profileManager.clearPinFailures(profileId)
    }

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

private object AndroidPinHasher {
    private const val ITERATIONS = 210_000
    private const val KEY_LENGTH_BITS = 256

    fun hash(pin: String): String {
        val salt = ByteArray(16).also(SecureRandom()::nextBytes)
        val derived = derive(pin, salt)
        return "pbkdf2-sha256:$ITERATIONS:${Base64.encodeToString(salt, Base64.NO_WRAP)}:${Base64.encodeToString(derived, Base64.NO_WRAP)}"
    }

    fun verify(pin: String, storedHash: String): Boolean {
        val parts = storedHash.split(':')
        if (parts.size != 4 || parts[0] != "pbkdf2-sha256") return false
        val iterations = parts[1].toIntOrNull() ?: return false
        val salt = runCatching { Base64.decode(parts[2], Base64.NO_WRAP) }.getOrNull() ?: return false
        val expected = runCatching { Base64.decode(parts[3], Base64.NO_WRAP) }.getOrNull() ?: return false
        return MessageDigest.isEqual(derive(pin, salt, iterations), expected)
    }

    private fun derive(pin: String, salt: ByteArray, iterations: Int = ITERATIONS): ByteArray =
        SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(PBEKeySpec(pin.toCharArray(), salt, iterations, KEY_LENGTH_BITS)).encoded
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
