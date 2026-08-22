package com.fluxa.app.shared.feature.profile

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import java.util.UUID

/** Common ProfileManager-backed CRUD used by Android and desktop stores. */
abstract class ProfileManagerPersistence(
    protected val profileManager: ProfileManager,
    private val initializeNewProfile: (UserProfile) -> UserProfile = { it },
) : ProfilePersistence {
    override suspend fun pinHash(profileId: String): String? =
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.pinHash

    override suspend fun canAttemptPin(profileId: String): Boolean =
        profileManager.canAttemptPin(profileId)

    override suspend fun recordPinFailure(profileId: String) {
        profileManager.recordPinFailure(profileId)
    }

    override suspend fun clearPinFailures(profileId: String) {
        profileManager.clearPinFailures(profileId)
    }

    override suspend fun activate(profileId: String) {
        profileManager.setLastActiveProfile(
            profileManager.getProfiles().firstOrNull { it.id == profileId },
        )
    }

    override suspend fun delete(profileId: String) {
        profileManager.deleteProfileById(profileId)
    }

    override suspend fun save(edit: ProfileEditUiModel, pinHash: String?): String {
        val existing = edit.id?.let { id ->
            profileManager.getProfiles().firstOrNull { it.id == id }
        }
        val profile = existing?.copy(
            profileName = edit.name,
            avatarUrl = edit.avatarUrl,
            pinHash = pinHash,
            biometricEnabled = edit.biometricEnabled,
        ) ?: initializeNewProfile(
            UserProfile(
                id = UUID.randomUUID().toString(),
                email = edit.name,
                profileName = edit.name,
                authKey = "",
                language = "en",
                avatarUrl = edit.avatarUrl,
                pinHash = pinHash,
                biometricEnabled = edit.biometricEnabled,
            ),
        )
        profileManager.saveProfile(profile)
        return profile.id
    }
}
