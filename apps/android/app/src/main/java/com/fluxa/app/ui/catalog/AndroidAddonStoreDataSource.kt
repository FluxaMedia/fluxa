package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.addonstore.InstalledAddonUiModel
import com.fluxa.app.ui.installLocalAddonForProfile
import com.fluxa.app.ui.moveLocalAddonForProfile
import com.fluxa.app.ui.removeLocalAddonForProfile
import com.fluxa.app.ui.setLocalAddonEnabledForProfile
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update

class AndroidAddonStoreDataSource(
    private val repository: StremioRepository,
    private val profileManager: ProfileManager,
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
) : AddonStoreDataSource {
    private data class Extras(
        val inputText: String = "",
        val detectedType: AddonStoreInputType = AddonStoreInputType.UNKNOWN,
        val isSubmittingInput: Boolean = false,
        val inputError: String? = null,
        val refreshingUrl: String? = null,
        val profileRevision: Long = 0L,
        val addedAddonName: String? = null
    )

    private val extras = MutableStateFlow(Extras())
    private val installedUserAddons = MutableStateFlow(emptyList<AddonDescriptor>() to false)

    override fun detectInputType(text: String): AddonStoreInputType = when (FluxaCoreNative.addonStoreInputType(text)) {
        "stremio_manifest" -> AddonStoreInputType.STREMIO_MANIFEST
        "search_query" -> AddonStoreInputType.SEARCH_QUERY
        else -> AddonStoreInputType.UNKNOWN
    }

    override fun observeAddonStore(): Flow<AddonStoreUiState> = combine(
        installedUserAddons,
        extras
    ) { userAddonsAndLoaded, ex ->
        buildState(
            userAddons = userAddonsAndLoaded.first,
            localLoaded = userAddonsAndLoaded.second,
            ex = ex,
        )
    }

    private fun buildState(
        userAddons: List<AddonDescriptor>,
        localLoaded: Boolean,
        ex: Extras
    ): AddonStoreUiState {
        val profile = activeProfile()
        val isNuvioProfile = !profile?.nuvioAccessToken.isNullOrBlank()
        val request = JsonObject().apply {
            add("repositoryAddons", Gson().toJsonTree(userAddons))
            add("localUrls", Gson().toJsonTree(profile?.safeInstalledLocalAddons.orEmpty()))
            add("disabledKeys", Gson().toJsonTree(profile?.disabledLocalAddons.orEmpty()))
            addProperty("isNuvioProfile", isNuvioProfile)
            addProperty("localLoaded", localLoaded)
            addProperty("refreshingUrl", ex.refreshingUrl)
        }
        val merged = JsonParser.parseString(FluxaCoreNative.addonStoreEntriesPlan(request.toString()))
            .asJsonArray
            .map { value ->
                val entry = value.asJsonObject
                val url = entry.get("url").asString
                InstalledAddonUiModel(
                    name = entry.get("name").asString,
                    description = entry.get("description").asString,
                    url = url,
                    logoUrl = entry.get("logoUrl")?.takeUnless { it.isJsonNull }?.asString,
                    version = entry.get("version")?.takeUnless { it.isJsonNull }?.asString,
                    configUrl = addonConfigUrl(url),
                    configurable = entry.get("configurable").asBoolean,
                    isEnabled = entry.get("isEnabled").asBoolean,
                    canRemove = entry.get("canRemove").asBoolean,
                    canMoveUp = entry.get("canMoveUp").asBoolean,
                    canMoveDown = entry.get("canMoveDown").asBoolean,
                    isRefreshing = entry.get("isRefreshing").asBoolean
                )
            }

        return AddonStoreUiState(
            installedAddons = merged,
            accentColorArgb = profile?.safeAccentColorArgb?.toLong()?.and(0xffffffffL) ?: 0xFF4CAF50L,
            isLoading = profile != null && !localLoaded,
            inputText = ex.inputText,
            inputDetectedType = ex.detectedType,
            isSubmittingInput = ex.isSubmittingInput,
            inputError = ex.inputError,
            addedAddonName = ex.addedAddonName
        )
    }

    override suspend fun refresh() {
        val profile = activeProfile()
        if (profile == null) {
            installedUserAddons.value = emptyList<AddonDescriptor>() to true
            return
        }
        if (!profile.nuvioAccessToken.isNullOrBlank()) {
            val result = runCatching { nuvioSyncCoordinator.fetchAddons(profile) }
                .getOrDefault(emptyList())
            installedUserAddons.value = result to true
            return
        }
        val result = runCatching {
            repository.getUserAddons(profile.authKey, profile.safeInstalledLocalAddons, forceRefresh = false)
        }.getOrDefault(emptyList())
        installedUserAddons.value = result to true
    }

    override suspend fun updateInputText(text: String) {
        extras.update {
            it.copy(
                inputText = text,
                inputError = null,
                detectedType = if (text.isBlank()) AddonStoreInputType.UNKNOWN else detectInputType(text)
            )
        }
    }

    override suspend fun submitInput(text: String) {
        val trimmed = text.trim()
        if (trimmed.isEmpty()) return
        when (detectInputType(trimmed)) {
            AddonStoreInputType.STREMIO_MANIFEST -> {
                extras.update { it.copy(isSubmittingInput = true, inputError = null) }
                val descriptor = repository.getAddonManifest(trimmed, forceRefresh = true)
                if (descriptor == null) {
                    extras.update {
                        it.copy(
                            isSubmittingInput = false,
                            inputError = AppStrings.t(activeProfile()?.language ?: "en", "addons.manifest_unreachable")
                        )
                    }
                } else {
                    installLocalAddonForProfile(activeProfile(), trimmed, profileManager, homeViewModel, onProfileChanged)
                    refreshProfileState()
                    extras.update {
                        it.copy(
                            inputText = "",
                            detectedType = AddonStoreInputType.UNKNOWN,
                            inputError = null,
                            isSubmittingInput = false,
                            addedAddonName = descriptor.manifest.name.takeIf { name -> name.isNotBlank() } ?: trimmed
                        )
                    }
                }
            }
            else -> Unit
        }
    }

    override suspend fun toggleAddon(url: String, enabled: Boolean) {
        val profile = activeProfile()
        val addon = installedUserAddons.value.first.firstOrNull { addonUrlIdentity(it.transportUrl) == addonUrlIdentity(url) }
        if (!profile?.nuvioAccessToken.isNullOrBlank() && addon?.isManaged == true) {
            nuvioSyncCoordinator.setAddonEnabled(profile, addon, enabled)
            refresh()
            return
        }
        setLocalAddonEnabledForProfile(activeProfile(), url, enabled, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    override suspend fun removeAddon(url: String) {
        val profile = activeProfile()
        val addon = installedUserAddons.value.first.firstOrNull { addonUrlIdentity(it.transportUrl) == addonUrlIdentity(url) }
        if (!profile?.nuvioAccessToken.isNullOrBlank() && addon?.isManaged == true) {
            nuvioSyncCoordinator.removeAddon(profile, addon)
            refresh()
            return
        }
        removeLocalAddonForProfile(activeProfile(), url, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    override suspend fun moveAddon(url: String, direction: Int) {
        val profile = activeProfile()
        val addon = installedUserAddons.value.first.firstOrNull { addonUrlIdentity(it.transportUrl) == addonUrlIdentity(url) }
        if (!profile?.nuvioAccessToken.isNullOrBlank() && addon?.isManaged == true) {
            nuvioSyncCoordinator.moveAddon(profile, addon, direction)
            refresh()
            return
        }
        moveLocalAddonForProfile(activeProfile(), url, direction, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    private fun refreshProfileState() {
        extras.update { it.copy(profileRevision = it.profileRevision + 1) }
    }

    override suspend fun refreshAddon(url: String) {
        extras.update { it.copy(refreshingUrl = url) }
        val profile = activeProfile()
        if (!profile?.nuvioAccessToken.isNullOrBlank()) {
            val reloaded = runCatching { nuvioSyncCoordinator.fetchAddons(profile) }.getOrDefault(emptyList())
            installedUserAddons.value = reloaded to true
            extras.update { it.copy(refreshingUrl = null) }
            return
        }
        val refreshed = repository.getAddonManifest(url, forceRefresh = true)
        val (current, loaded) = installedUserAddons.value
        installedUserAddons.value = if (refreshed != null) {
            (current.filterNot { addonUrlIdentity(it.transportUrl) == addonUrlIdentity(url) } + refreshed) to loaded
        } else {
            val profile = activeProfile()
            val reloaded = profile?.let {
                repository.getUserAddons(it.authKey, it.safeInstalledLocalAddons, forceRefresh = true)
            }.orEmpty()
            reloaded to loaded
        }
        extras.update { it.copy(refreshingUrl = null) }
    }

    override suspend fun dismissAddedAddonDialog() {
        extras.update { it.copy(addedAddonName = null) }
    }
}
