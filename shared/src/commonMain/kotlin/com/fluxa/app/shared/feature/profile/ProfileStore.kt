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
