package com.fluxa.app.shared.feature.profile

import com.fluxa.app.common.PinHasher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
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
            ProfileAction.PickerSettingsRequested -> Unit
            ProfileAction.PickerSettingsClosed -> Unit
            is ProfileAction.BackgroundUrlChanged -> dataSource.setPickerBackground(action.url)
            is ProfileAction.AddAvatarPackRequested -> dataSource.addAvatarPack(action.repositoryUrl)
            is ProfileAction.RemoveAvatarPackRequested -> dataSource.removeAvatarPack(action.packId)
            is ProfileAction.RefreshAvatarPackRequested -> dataSource.refreshAvatarPack(action.repositoryUrl)
        }
    }

    suspend fun deleteProfile(id: String) = dataSource.deleteProfile(id)

    suspend fun saveProfile(edit: ProfileEditUiModel): String = dataSource.saveProfile(edit)
}

data class ProfileStoreSnapshot(
    val activeProfile: ProfileUiModel? = null,
    val profiles: List<ProfileUiModel> = emptyList(),
    val isLoading: Boolean = false,
    val pickerBackgroundUrl: String? = null,
    val avatarPacks: List<ProfileAvatarPackUiModel> = emptyList()
)

interface ProfilePersistence {
    fun observe(): Flow<ProfileStoreSnapshot>
    suspend fun pinHash(profileId: String): String?
    suspend fun activate(profileId: String)
    suspend fun delete(profileId: String)
    suspend fun save(edit: ProfileEditUiModel, pinHash: String?): String
    suspend fun setPickerBackground(url: String?) {}
    suspend fun addAvatarPack(repositoryUrl: String): Result<Unit> = Result.success(Unit)
    suspend fun removeAvatarPack(packId: String) {}
    suspend fun refreshAvatarPack(repositoryUrl: String): Result<Unit> = Result.success(Unit)
}

class SharedProfileDataSource(
    private val store: ProfilePersistence
) : ProfileDataSource {
    private val pendingPinId = MutableStateFlow<String?>(null)
    private val pinError = MutableStateFlow(false)
    private val avatarPackDiscoveryInProgress = MutableStateFlow(false)
    private val avatarPackDiscoveryError = MutableStateFlow(false)

    override fun observeProfiles(): Flow<ProfileUiState> = combine(
        store.observe(), pendingPinId, pinError, avatarPackDiscoveryInProgress, avatarPackDiscoveryError
    ) { snapshot, pendingId, error, discoveryInProgress, discoveryError ->
        ProfileUiState(
            activeProfile = snapshot.activeProfile,
            profiles = snapshot.profiles,
            isLoading = snapshot.isLoading,
            pendingPinProfile = snapshot.profiles.firstOrNull { it.id == pendingId },
            pinError = error,
            pickerBackgroundUrl = snapshot.pickerBackgroundUrl,
            avatarPacks = snapshot.avatarPacks,
            avatarPackDiscoveryInProgress = discoveryInProgress,
            avatarPackDiscoveryError = discoveryError
        )
    }

    override suspend fun selectProfile(id: String) {
        if (store.pinHash(id).isNullOrBlank()) {
            store.activate(id)
        } else {
            pendingPinId.value = id
            pinError.value = false
        }
    }

    override suspend fun attemptPin(profileId: String, pin: String) {
        if (PinHasher.hash(pin) == store.pinHash(profileId)) {
            store.activate(profileId)
            pendingPinId.value = null
            pinError.value = false
        } else {
            pinError.value = true
        }
    }

    override suspend fun confirmBiometricUnlock(profileId: String) {
        store.activate(profileId)
        pendingPinId.value = null
        pinError.value = false
    }

    override suspend fun cancelPinUnlock() {
        pendingPinId.value = null
        pinError.value = false
    }

    override suspend fun deleteProfile(id: String) = store.delete(id)

    override suspend fun saveProfile(edit: ProfileEditUiModel): String {
        val pinHash = when {
            edit.newPin != null -> PinHasher.hash(edit.newPin)
            edit.keepExistingPin && edit.id != null -> store.pinHash(edit.id)
            else -> null
        }
        return store.save(edit, pinHash)
    }

    override suspend fun setPickerBackground(url: String?) = store.setPickerBackground(url)

    override suspend fun addAvatarPack(repositoryUrl: String) {
        avatarPackDiscoveryInProgress.value = true
        avatarPackDiscoveryError.value = false
        val result = store.addAvatarPack(repositoryUrl)
        avatarPackDiscoveryInProgress.value = false
        avatarPackDiscoveryError.value = result.isFailure
    }

    override suspend fun removeAvatarPack(packId: String) = store.removeAvatarPack(packId)

    override suspend fun refreshAvatarPack(repositoryUrl: String) {
        avatarPackDiscoveryError.value = avatarPackDiscoveryError.value || store.refreshAvatarPack(repositoryUrl).isFailure
    }

    override suspend fun refreshAllAvatarPacks() {
        store.observe().first().avatarPacks.map { it.repositoryUrl }.distinct().forEach { repositoryUrl ->
            store.refreshAvatarPack(repositoryUrl)
        }
    }
}
