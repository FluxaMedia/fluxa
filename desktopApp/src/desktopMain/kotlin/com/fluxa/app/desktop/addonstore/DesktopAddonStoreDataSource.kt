package com.fluxa.app.desktop.addonstore

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.addonstore.InstalledAddonUiModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class DesktopAddonStoreDataSource(
    private val addonRepository: AddonRepository,
    private val registry: DesktopAddonRegistry,
    private val language: String? = null
) : AddonStoreDataSource {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val state = MutableStateFlow(AddonStoreUiState())

    init {
        scope.launch { reload() }
    }

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

    override suspend fun refresh() = reload()

    override suspend fun updateInputText(text: String) {
        state.value = state.value.copy(inputText = text, inputDetectedType = detectInputType(text))
    }

    override suspend fun submitInput(text: String) {
        val trimmed = text.trim()
        if (detectInputType(trimmed) != AddonStoreInputType.STREMIO_MANIFEST) {
            state.value = state.value.copy(inputError = AppStrings.t(language, "addons.manifest_unreachable"))
            return
        }
        state.value = state.value.copy(isSubmittingInput = true, inputError = null)
        val descriptor = runCatching { addonRepository.getAddonManifest(trimmed, forceRefresh = true) }.getOrNull()
        if (descriptor == null) {
            state.value = state.value.copy(isSubmittingInput = false, inputError = AppStrings.t(language, "addons.manifest_unreachable"))
            return
        }
        registry.addUrl(trimmed)
        state.value = state.value.copy(
            isSubmittingInput = false,
            inputText = "",
            inputDetectedType = AddonStoreInputType.UNKNOWN,
            addedAddonName = descriptor.manifest.name
        )
        reload()
    }

    override suspend fun toggleAddon(url: String, enabled: Boolean) {
        registry.setEnabled(url, enabled)
        reload()
    }

    override suspend fun removeAddon(url: String) {
        registry.removeUrl(url)
        reload()
    }

    override suspend fun moveAddon(url: String, direction: Int) {
        registry.moveUrl(url, direction)
        reload()
    }

    override suspend fun refreshAddon(url: String) {
        runCatching { addonRepository.getAddonManifest(url, forceRefresh = true) }
        reload()
    }

    override suspend fun dismissAddedAddonDialog() {
        state.value = state.value.copy(addedAddonName = null)
    }

    private suspend fun reload() {
        state.value = state.value.copy(isLoading = true)
        val urls = registry.installedUrls()
        val disabled = registry.disabledUrls()
        val models = coroutineScope {
            urls.mapIndexed { index, url ->
                async {
                    val descriptor = runCatching { addonRepository.getAddonManifest(url) }.getOrNull()
                    InstalledAddonUiModel(
                        name = descriptor?.manifest?.name?.takeIf { it.isNotBlank() } ?: url,
                        description = descriptor?.manifest?.description.orEmpty(),
                        url = url,
                        logoUrl = descriptor?.manifest?.logo,
                        version = descriptor?.manifest?.version,
                        configurable = descriptor?.manifest?.configurable == true,
                        isEnabled = url !in disabled,
                        canRemove = true,
                        canMoveUp = index > 0,
                        canMoveDown = index < urls.lastIndex
                    )
                }
            }.awaitAll()
        }
        state.value = state.value.copy(installedAddons = models, isLoading = false)
    }
}
