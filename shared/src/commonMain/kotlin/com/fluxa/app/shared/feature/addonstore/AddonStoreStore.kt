package com.fluxa.app.shared.feature.addonstore

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class AddonStoreStore(
    private val dataSource: AddonStoreDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<AddonStoreUiState> = dataSource.observeAddonStore()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), AddonStoreUiState(isLoading = true))

    suspend fun dispatch(action: AddonStoreAction) {
        when (action) {
            AddonStoreAction.Refresh -> dataSource.refresh()
            is AddonStoreAction.InputChanged -> dataSource.updateInputText(action.text)
            AddonStoreAction.SubmitInput -> dataSource.submitInput(state.value.inputText)
            is AddonStoreAction.AddonToggled -> dataSource.toggleAddon(action.url, action.enabled)
            is AddonStoreAction.AddonRemoved -> dataSource.removeAddon(action.url)
            is AddonStoreAction.AddonMoved -> dataSource.moveAddon(action.url, action.direction)
            is AddonStoreAction.AddonRefreshed -> dataSource.refreshAddon(action.url)
            is AddonStoreAction.ConfigureRequested -> Unit
            is AddonStoreAction.RepoOpened -> dataSource.openRepo(action.url)
            AddonStoreAction.RepoDialogDismissed -> dataSource.dismissRepoDialog()
            is AddonStoreAction.RepoRemoved -> dataSource.removeRepo(action.url)
            is AddonStoreAction.RepoPluginToggled -> dataSource.toggleRepoPlugin(action.repoUrl, action.internalName)
        }
    }
}
