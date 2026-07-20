package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleAddonStoreActionSnapshot(
    val type: String,
    val url: String? = null,
    val text: String? = null,
    val enabled: Boolean? = null,
    val direction: Int? = null
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

    override suspend fun dismissAddedAddonDialog() {
        state.value = state.value.copy(addedAddonName = null)
    }

    fun setOnActionRequested(handler: (AppleAddonStoreActionSnapshot) -> Unit) {
        onActionRequested = handler
    }

    private fun postAction(action: AppleAddonStoreActionSnapshot) {
        onActionRequested(action)
    }
}
