package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.addonstore.CloudstreamPluginUiModel
import com.fluxa.app.shared.feature.addonstore.CloudstreamRepoUiModel
import com.fluxa.app.shared.feature.addonstore.InstalledAddonUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleAddonStoreActionSnapshot(
    val type: String,
    val url: String? = null,
    val text: String? = null,
    val enabled: Boolean? = null,
    val direction: Int? = null,
    val repoUrl: String? = null,
    val internalName: String? = null
)

class AppleAddonStoreDataSource : AddonStoreDataSource {
    private val state = MutableStateFlow(AddonStoreUiState())
    private var onActionRequested: (AppleAddonStoreActionSnapshot) -> Unit = {}

    override fun observeAddonStore(): Flow<AddonStoreUiState> = state.asStateFlow()

    override fun detectInputType(text: String): AddonStoreInputType {
        val trimmed = text.trim()
        return when {
            trimmed.isEmpty() -> AddonStoreInputType.UNKNOWN
            trimmed.endsWith("manifest.json") || trimmed.startsWith("stremio://") -> AddonStoreInputType.STREMIO_MANIFEST
            trimmed.endsWith(".json") && trimmed.startsWith("http") -> AddonStoreInputType.CLOUDSTREAM_REPO
            trimmed.startsWith("http://") || trimmed.startsWith("https://") -> AddonStoreInputType.STREMIO_MANIFEST
            else -> AddonStoreInputType.SEARCH_QUERY
        }
    }

    override suspend fun refresh() {
        state.value = state.value.copy(isLoading = true)
        postAction(AppleAddonStoreActionSnapshot(type = "refresh"))
    }

    override suspend fun updateInputText(text: String) {
        state.value = state.value.copy(inputText = text, inputDetectedType = detectInputType(text))
    }

    override suspend fun submitInput(text: String) {
        state.value = state.value.copy(isSubmittingInput = true)
        postAction(AppleAddonStoreActionSnapshot(type = "submitInput", text = text))
    }

    override suspend fun toggleAddon(url: String, enabled: Boolean) {
        postAction(AppleAddonStoreActionSnapshot(type = "toggleAddon", url = url, enabled = enabled))
    }

    override suspend fun removeAddon(url: String) {
        postAction(AppleAddonStoreActionSnapshot(type = "removeAddon", url = url))
    }

    override suspend fun moveAddon(url: String, direction: Int) {
        postAction(AppleAddonStoreActionSnapshot(type = "moveAddon", url = url, direction = direction))
    }

    override suspend fun refreshAddon(url: String) {
        postAction(AppleAddonStoreActionSnapshot(type = "refreshAddon", url = url))
    }

    override suspend fun openRepo(url: String) {
        state.value = state.value.copy(openRepoUrl = url, isLoadingRepoPlugins = true)
        postAction(AppleAddonStoreActionSnapshot(type = "openRepo", url = url))
    }

    override suspend fun dismissRepoDialog() {
        state.value = state.value.copy(openRepoUrl = null, openRepoName = null, openRepoPlugins = emptyList(), repoDialogError = null)
    }

    override suspend fun removeRepo(url: String) {
        postAction(AppleAddonStoreActionSnapshot(type = "removeRepo", url = url))
    }

    override suspend fun toggleRepoPlugin(repoUrl: String, internalName: String) {
        postAction(AppleAddonStoreActionSnapshot(type = "toggleRepoPlugin", repoUrl = repoUrl, internalName = internalName))
    }

    fun setOnActionRequested(handler: (AppleAddonStoreActionSnapshot) -> Unit) {
        onActionRequested = handler
    }

    private fun postAction(action: AppleAddonStoreActionSnapshot) {
        onActionRequested(action)
    }
}
