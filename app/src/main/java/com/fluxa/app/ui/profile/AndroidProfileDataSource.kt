package com.fluxa.app.ui.profile

import android.content.SharedPreferences
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.profile.SettingsUiState
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

class AndroidProfileDataSource(
    private val profileManager: ProfileManager
) : ProfileDataSource {
    override fun observeProfiles(): Flow<ProfileUiState> = profilesFlow()

    override fun observeSettings(profileId: String): Flow<SettingsUiState> = callbackFlow {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ ->
            profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { trySend(it.toSettingsUiState()) }
        }
        profileManager.registerOnChangeListener(listener)
        profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { trySend(it.toSettingsUiState()) }
        awaitClose { profileManager.unregisterOnChangeListener(listener) }
    }

    override suspend fun selectProfile(id: String) {
        profileManager.setLastActiveProfile(profileManager.getProfiles().firstOrNull { it.id == id })
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
    accentColorArgb = safeAccentColorArgb.toLong() and 0xffffffffL
)

private fun UserProfile.toSettingsUiState(): SettingsUiState = SettingsUiState(
    language = safeLanguage,
    cardLayout = safeCardLayout,
    autoPlayNextEpisode = safeAutoPlayNextEpisode,
    subtitlesEnabled = safeAutoEnableSubtitles,
    preferredAudioLanguage = safePreferredAudioLanguage,
    preferredSubtitleLanguage = safePreferredSubtitleLanguage
)
