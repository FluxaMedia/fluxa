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
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import platform.Foundation.NSNotificationCenter

class AppleAddonStoreDataSource : AddonStoreDataSource {
    private val state = MutableStateFlow(AddonStoreUiState())

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
        postAction("FluxaAppleAddonStoreRequested") { }
    }

    override suspend fun updateInputText(text: String) {
        state.value = state.value.copy(inputText = text, inputDetectedType = detectInputType(text))
    }

    override suspend fun submitInput(text: String) {
        state.value = state.value.copy(isSubmittingInput = true)
        postAction("FluxaAppleAddonStoreSubmitInputRequested") { put("text", text) }
    }

    override suspend fun toggleAddon(url: String, enabled: Boolean) {
        postAction("FluxaAppleAddonToggleRequested") {
            put("url", url)
            put("enabled", enabled)
        }
    }

    override suspend fun removeAddon(url: String) {
        postAction("FluxaAppleAddonRemoveRequested") { put("url", url) }
    }

    override suspend fun moveAddon(url: String, direction: Int) {
        postAction("FluxaAppleAddonMoveRequested") {
            put("url", url)
            put("direction", direction)
        }
    }

    override suspend fun refreshAddon(url: String) {
        postAction("FluxaAppleAddonRefreshRequested") { put("url", url) }
    }

    override suspend fun openRepo(url: String) {
        state.value = state.value.copy(openRepoUrl = url, isLoadingRepoPlugins = true)
        postAction("FluxaAppleAddonRepoOpenRequested") { put("url", url) }
    }

    override suspend fun dismissRepoDialog() {
        state.value = state.value.copy(openRepoUrl = null, openRepoName = null, openRepoPlugins = emptyList(), repoDialogError = null)
    }

    override suspend fun removeRepo(url: String) {
        postAction("FluxaAppleAddonRepoRemoveRequested") { put("url", url) }
    }

    override suspend fun toggleRepoPlugin(repoUrl: String, internalName: String) {
        postAction("FluxaAppleAddonRepoPluginToggleRequested") {
            put("repoUrl", repoUrl)
            put("internalName", internalName)
        }
    }

    private fun postAction(name: String, extra: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit) {
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = name,
            `object` = buildJsonObject(extra).toString(),
            userInfo = null
        )
    }

    fun updateJson(addonStoreJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(addonStoreJson).jsonObject
            AddonStoreUiState(
                installedAddons = root["installedAddons"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toInstalledAddon() },
                cloudstreamRepos = root["cloudstreamRepos"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toRepo() },
                accentColorArgb = root["accentColorArgb"]?.jsonPrimitive?.contentOrNull?.toLongOrNull() ?: 0xFF4CAF50,
                isLoading = root.boolean("isLoading"),
                inputText = root.string("inputText").orEmpty(),
                isSubmittingInput = root.boolean("isSubmittingInput"),
                inputError = root.string("inputError"),
                openRepoUrl = root.string("openRepoUrl"),
                openRepoName = root.string("openRepoName"),
                openRepoPlugins = root["openRepoPlugins"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toPlugin() },
                isLoadingRepoPlugins = root.boolean("isLoadingRepoPlugins"),
                repoDialogError = root.string("repoDialogError")
            )
        }.getOrElse { state.value }
    }
}

private fun Map<String, JsonElement>.toInstalledAddon(): InstalledAddonUiModel? {
    val name = string("name") ?: return null
    val url = string("url") ?: return null
    return InstalledAddonUiModel(
        name = name,
        description = string("description").orEmpty(),
        url = url,
        logoUrl = string("logoUrl"),
        version = string("version"),
        configUrl = string("configUrl"),
        configurable = boolean("configurable"),
        isEnabled = get("isEnabled")?.jsonPrimitive?.contentOrNull?.toBooleanStrictOrNull() ?: true,
        canRemove = boolean("canRemove"),
        canMoveUp = boolean("canMoveUp"),
        canMoveDown = boolean("canMoveDown"),
        isRefreshing = boolean("isRefreshing")
    )
}

private fun Map<String, JsonElement>.toRepo(): CloudstreamRepoUiModel? {
    val name = string("name") ?: return null
    val url = string("url") ?: return null
    return CloudstreamRepoUiModel(name = name, url = url, iconUrl = string("iconUrl"))
}

private fun Map<String, JsonElement>.toPlugin(): CloudstreamPluginUiModel? {
    val internalName = string("internalName") ?: return null
    return CloudstreamPluginUiModel(
        internalName = internalName,
        name = string("name").orEmpty(),
        description = string("description").orEmpty(),
        iconUrl = string("iconUrl"),
        typesLabel = string("typesLabel").orEmpty(),
        isInstalled = boolean("isInstalled")
    )
}

private fun Map<String, JsonElement>.string(key: String): String? =
    get(key)?.jsonPrimitive?.contentOrNull

private fun Map<String, JsonElement>.boolean(key: String): Boolean =
    string(key)?.toBooleanStrictOrNull() ?: false
