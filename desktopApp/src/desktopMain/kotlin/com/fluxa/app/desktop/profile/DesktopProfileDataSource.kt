package com.fluxa.app.desktop.profile

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAccentColorArgb
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfilePersistence
import com.fluxa.app.shared.feature.profile.ProfileStoreSnapshot
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.SharedProfileDataSource
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.Base64
import java.util.UUID
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class DesktopProfileDataSource(
    profileManager: ProfileManager
) : ProfileDataSource by SharedProfileDataSource(DesktopProfileStore(profileManager))

private class DesktopProfileStore(
    private val profileManager: ProfileManager
) : ProfilePersistence {
    private val state = MutableStateFlow(snapshot())

    init {
        profileManager.addChangeListener { state.value = snapshot() }
    }

    private fun snapshot(): ProfileStoreSnapshot {
        val profiles = profileManager.getProfiles()
        val activeId = profileManager.getLastActiveProfileId()
        return ProfileStoreSnapshot(
            activeProfile = profiles.firstOrNull { it.id == activeId }?.toUiModel(),
            profiles = profiles.map(UserProfile::toUiModel)
        )
    }

    override fun observe(): Flow<ProfileStoreSnapshot> = state.asStateFlow()

    override suspend fun pinHash(profileId: String): String? =
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.pinHash

    override suspend fun createPinHash(pin: String): String = DesktopPinHasher.hash(pin)

    override suspend fun verifyPin(profileId: String, pin: String, storedHash: String): Boolean =
        DesktopPinHasher.verify(pin, storedHash)

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
            biometricEnabled = edit.biometricEnabled
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

private object DesktopPinHasher {
    private const val ITERATIONS = 210_000
    private const val KEY_LENGTH_BITS = 256
    private const val ALGORITHM_PREFIX = "pbkdf2-sha256"

    fun hash(pin: String): String {
        val salt = ByteArray(16).also(SecureRandom()::nextBytes)
        val derived = derive(pin, salt, ITERATIONS)
        val encoder = Base64.getEncoder()
        return "$ALGORITHM_PREFIX:$ITERATIONS:${encoder.encodeToString(salt)}:${encoder.encodeToString(derived)}"
    }

    fun verify(pin: String, storedHash: String): Boolean {
        val parts = storedHash.split(':')
        if (parts.size != 4 || parts[0] != ALGORITHM_PREFIX) return false
        val iterations = parts[1].toIntOrNull() ?: return false
        val decoder = Base64.getDecoder()
        val salt = runCatching { decoder.decode(parts[2]) }.getOrNull() ?: return false
        val expected = runCatching { decoder.decode(parts[3]) }.getOrNull() ?: return false
        return MessageDigest.isEqual(derive(pin, salt, iterations), expected)
    }

    private fun derive(pin: String, salt: ByteArray, iterations: Int): ByteArray =
        SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256")
            .generateSecret(PBEKeySpec(pin.toCharArray(), salt, iterations, KEY_LENGTH_BITS))
            .encoded
}
