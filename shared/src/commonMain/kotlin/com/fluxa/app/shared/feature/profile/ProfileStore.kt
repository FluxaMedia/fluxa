package com.fluxa.app.shared.feature.profile

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class ProfileStore(
    private val dataSource: ProfileDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<ProfileUiState> = dataSource.observeProfiles()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), ProfileUiState(isLoading = true))

    suspend fun selectProfile(profile: ProfileUiModel) {
        dataSource.selectProfile(profile.id)
    }

    suspend fun dispatch(action: ProfileAction) {
        when (action) {
            is ProfileAction.Selected -> dataSource.selectProfile(action.profile.id)
            ProfileAction.AddRequested -> Unit
            is ProfileAction.EditRequested -> Unit
            is ProfileAction.PinAttempt -> dataSource.attemptPin(action.profileId, action.pin)
            ProfileAction.PinCancelled -> dataSource.cancelPinUnlock()
        }
    }

    suspend fun deleteProfile(id: String) = dataSource.deleteProfile(id)

    suspend fun saveProfile(edit: ProfileEditUiModel): String = dataSource.saveProfile(edit)
}

class ProfileSettingsStore(
    private val profileId: String,
    private val dataSource: ProfileDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<SettingsUiState> = dataSource.observeSettings(profileId)
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), SettingsUiState())

    suspend fun update(settings: SettingsUiState) {
        dataSource.updateSettings(profileId, settings)
    }
}
