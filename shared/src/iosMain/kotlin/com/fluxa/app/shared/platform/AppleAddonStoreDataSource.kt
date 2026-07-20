package com.fluxa.app.shared.platform

import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.addonstore.InstalledAddonUiModel
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

data class AppleInstalledAddonSnapshot(
    val name: String,
    val description: String,
    val url: String,
    val logoUrl: String?,
    val version: String?,
    val configurable: Boolean,
    val isEnabled: Boolean,
    val canMoveUp: Boolean,
    val canMoveDown: Boolean,
    val isRefreshing: Boolean
)

data class AppleAddonStoreSnapshot(
    val installedAddons: List<AppleInstalledAddonSnapshot> = emptyList(),
    val isLoading: Boolean = false,
    val isSubmittingInput: Boolean = false,
    val inputFailed: Boolean = false,
    val addedAddonName: String? = null,
    val clearInputOnSuccess: Boolean = false
)

private fun AppleInstalledAddonSnapshot.toUiModel(): InstalledAddonUiModel = InstalledAddonUiModel(
    name = name,
    description = description,
    url = url,
    logoUrl = logoUrl,
    version = version,
    configUrl = null,
    configurable = configurable,
    isEnabled = isEnabled,
    canRemove = true,
    canMoveUp = canMoveUp,
    canMoveDown = canMoveDown,
    isRefreshing = isRefreshing
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

    fun update(snapshot: AppleAddonStoreSnapshot) {
        state.value = state.value.copy(
            installedAddons = snapshot.installedAddons.map { it.toUiModel() },
            isLoading = snapshot.isLoading,
            isSubmittingInput = snapshot.isSubmittingInput,
            inputError = if (snapshot.inputFailed) AppStrings.t("en", "addons.manifest_unreachable") else null,
            addedAddonName = snapshot.addedAddonName,
            inputText = if (snapshot.clearInputOnSuccess) "" else state.value.inputText,
            inputDetectedType = if (snapshot.clearInputOnSuccess) AddonStoreInputType.UNKNOWN else state.value.inputDetectedType
        )
    }

    private fun postAction(action: AppleAddonStoreActionSnapshot) {
        onActionRequested(action)
    }
}
