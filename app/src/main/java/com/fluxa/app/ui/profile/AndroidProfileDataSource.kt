package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.PinHasher
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.profile.SettingsUiState
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine

class AndroidProfileDataSource(
    private val profileManager: ProfileManager
) : ProfileDataSource {

    private val pendingPinId = MutableStateFlow<String?>(null)
    private val pinError = MutableStateFlow(false)

    override fun observeProfiles(): Flow<ProfileUiState> = combine(profilesFlow(), pendingPinId, pinError) { state, pendingId, error ->
        state.copy(
            pendingPinProfile = state.profiles.firstOrNull { it.id == pendingId },
            pinError = error
        )
    }

    override fun observeSettings(profileId: String): Flow<SettingsUiState> = callbackFlow {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ ->
            profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { trySend(it.toSettingsUiState()) }
        }
        profileManager.registerOnChangeListener(listener)
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { trySend(it.toSettingsUiState()) }
        awaitClose { profileManager.unregisterOnChangeListener(listener) }
    }

    override suspend fun selectProfile(id: String) {
        val target = profileManager.getProfiles().firstOrNull { it.id == id } ?: return
        if (target.pinHash.isNullOrBlank()) {
            profileManager.setLastActiveProfile(target)
        } else {
            pendingPinId.value = target.id
            pinError.value = false
        }
    }

    override suspend fun attemptPin(profileId: String, pin: String) {
        val target = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return
        if (PinHasher.hash(pin) == target.pinHash) {
            profileManager.setLastActiveProfile(target)
            pendingPinId.value = null
            pinError.value = false
        } else {
            pinError.value = true
        }
    }

    override suspend fun confirmBiometricUnlock(profileId: String) {
        val target = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return
        profileManager.setLastActiveProfile(target)
        pendingPinId.value = null
        pinError.value = false
    }

    override suspend fun cancelPinUnlock() {
        pendingPinId.value = null
        pinError.value = false
    }

    override suspend fun deleteProfile(id: String) {
        profileManager.deleteProfileById(id)
    }

    override suspend fun saveProfile(edit: ProfileEditUiModel): String {
        val existing = edit.id?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
        val newPin = edit.newPin
        val pinHash = when {
            newPin != null -> PinHasher.hash(newPin)
            edit.keepExistingPin -> existing?.pinHash
            else -> null
        }
        val profile = existing?.copy(
            profileName = edit.name,
            avatarUrl = edit.avatarUrl,
            pinHash = pinHash,
            biometricEnabled = edit.biometricEnabled
        ) ?: UserProfile(
            id = java.util.UUID.randomUUID().toString(),
            email = edit.name,
            profileName = edit.name,
            authKey = "",
            isGuest = false,
            language = "en",
            avatarUrl = edit.avatarUrl,
            pinHash = pinHash,
            biometricEnabled = edit.biometricEnabled,
            localAddons = listOf("https://v3-cinemeta.strem.io/manifest.json")
        )
        profileManager.saveProfile(profile)
        return profile.id
    }

    override suspend fun updateSettings(profileId: String, settings: SettingsUiState) {
        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return
        profileManager.saveProfile(profile.copy(
            language = settings.language,
            cardLayout = settings.cardLayout,
            autoPlayNextEpisode = settings.autoPlayNextEpisode,
            autoEnableSubtitles = settings.subtitlesEnabled,
            preferredAudioLanguage = settings.preferredAudioLanguage,
            preferredSubtitleLanguage = settings.preferredSubtitleLanguage
        ))
    }

    private fun profilesFlow(): Flow<ProfileUiState> = callbackFlow {
        fun emit() {
            val profiles = profileManager.getProfiles()
            val activeId = profileManager.getLastActiveProfileId()
            trySend(ProfileUiState(
                activeProfile = profiles.firstOrNull { it.id == activeId }?.toProfileUiModel(),
                profiles = profiles.map { it.toProfileUiModel() }
            ))
        }
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> emit() }
        profileManager.registerOnChangeListener(listener)
        emit()
        awaitClose { profileManager.unregisterOnChangeListener(listener) }
    }
}

private fun UserProfile.toProfileUiModel(): ProfileUiModel = ProfileUiModel(
    id = id,
    name = profileName?.takeIf { it.isNotBlank() } ?: email,
    avatarUrl = avatarUrl,
    language = safeLanguage,
    accentColorArgb = safeAccentColorArgb.toLong() and 0xffffffffL,
    hasPin = !pinHash.isNullOrBlank(),
    biometricEnabled = biometricEnabled == true
)

private fun UserProfile.toSettingsUiState(): SettingsUiState = SettingsUiState(
    language = safeLanguage,
    cardLayout = safeCardLayout,
    autoPlayNextEpisode = safeAutoPlayNextEpisode,
    subtitlesEnabled = safeAutoEnableSubtitles,
    preferredAudioLanguage = safePreferredAudioLanguage,
    preferredSubtitleLanguage = safePreferredSubtitleLanguage
)
