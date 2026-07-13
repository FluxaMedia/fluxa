package com.fluxa.app.shared.feature.addonstore

import kotlinx.coroutines.flow.Flow

enum class AddonStoreInputType {
    UNKNOWN,
    STREMIO_MANIFEST,
    CLOUDSTREAM_REPO,
    SEARCH_QUERY
}

data class InstalledAddonUiModel(
    val name: String,
    val description: String,
    val url: String,
    val logoUrl: String? = null,
    val version: String? = null,
    val configUrl: String? = null,
    val configurable: Boolean = false,
    val isEnabled: Boolean = true,
    val canRemove: Boolean = false,
    val canMoveUp: Boolean = false,
    val canMoveDown: Boolean = false,
    val isRefreshing: Boolean = false
)

data class CloudstreamRepoUiModel(
    val name: String,
    val url: String,
    val iconUrl: String? = null
)

data class CloudstreamPluginUiModel(
    val internalName: String,
    val name: String,
    val description: String,
    val iconUrl: String? = null,
    val typesLabel: String = "",
    val isInstalled: Boolean = false
)

data class AddonStoreUiState(
    val installedAddons: List<InstalledAddonUiModel> = emptyList(),
    val cloudstreamRepos: List<CloudstreamRepoUiModel> = emptyList(),
    val accentColorArgb: Long = 0xFF4CAF50,
    val isLoading: Boolean = false,
    val inputText: String = "",
    val inputDetectedType: AddonStoreInputType = AddonStoreInputType.UNKNOWN,
    val isSubmittingInput: Boolean = false,
    val inputError: String? = null,
    val openRepoUrl: String? = null,
    val openRepoName: String? = null,
    val openRepoPlugins: List<CloudstreamPluginUiModel> = emptyList(),
    val isLoadingRepoPlugins: Boolean = false,
    val installingPluginKeys: Set<String> = emptySet(),
    val repoDialogError: String? = null
)

sealed interface AddonStoreAction {
    data object Refresh : AddonStoreAction
    data class InputChanged(val text: String) : AddonStoreAction
    data object SubmitInput : AddonStoreAction
    data class AddonToggled(val url: String, val enabled: Boolean) : AddonStoreAction
    data class AddonRemoved(val url: String) : AddonStoreAction
    data class AddonMoved(val url: String, val direction: Int) : AddonStoreAction
    data class AddonRefreshed(val url: String) : AddonStoreAction
    data class ConfigureRequested(val url: String) : AddonStoreAction
    data class RepoOpened(val url: String) : AddonStoreAction
    data object RepoDialogDismissed : AddonStoreAction
    data class RepoRemoved(val url: String) : AddonStoreAction
    data class RepoPluginToggled(val repoUrl: String, val internalName: String) : AddonStoreAction
}

interface AddonStoreDataSource {
    fun observeAddonStore(): Flow<AddonStoreUiState>
    fun detectInputType(text: String): AddonStoreInputType
    suspend fun refresh()
    suspend fun updateInputText(text: String)
    suspend fun submitInput(text: String)
    suspend fun toggleAddon(url: String, enabled: Boolean)
    suspend fun removeAddon(url: String)
    suspend fun moveAddon(url: String, direction: Int)
    suspend fun refreshAddon(url: String)
    suspend fun openRepo(url: String)
    suspend fun dismissRepoDialog()
    suspend fun removeRepo(url: String)
    suspend fun toggleRepoPlugin(repoUrl: String, internalName: String)
}
