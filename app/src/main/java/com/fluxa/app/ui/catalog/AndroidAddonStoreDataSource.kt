package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreInputType
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.addonstore.CloudstreamPluginUiModel
import com.fluxa.app.shared.feature.addonstore.CloudstreamRepoUiModel
import com.fluxa.app.shared.feature.addonstore.InstalledAddonUiModel
import com.fluxa.app.ui.installLocalAddonForProfile
import com.fluxa.app.ui.moveLocalAddonForProfile
import com.fluxa.app.ui.removeLocalAddonForProfile
import com.fluxa.app.ui.setLocalAddonEnabledForProfile
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update

class AndroidAddonStoreDataSource(
    private val pluginManager: PluginManager,
    private val repository: StremioRepository,
    private val profileManager: ProfileManager,
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit
) : AddonStoreDataSource {

    private data class Extras(
        val inputText: String = "",
        val detectedType: AddonStoreInputType = AddonStoreInputType.UNKNOWN,
        val isSubmittingInput: Boolean = false,
        val inputError: String? = null,
        val refreshingUrl: String? = null,
        val openRepoUrl: String? = null,
        val openRepoName: String? = null,
        val openRepoPlugins: List<PluginInfo> = emptyList(),
        val isLoadingRepoPlugins: Boolean = false,
        val installingPluginKeys: Set<String> = emptySet(),
        val repoDialogError: String? = null,
        val profileRevision: Long = 0L
    )

    private val extras = MutableStateFlow(Extras())
    private val installedUserAddons = MutableStateFlow(emptyList<AddonDescriptor>() to false)

    override fun detectInputType(text: String): AddonStoreInputType = when (FluxaCoreNative.addonStoreInputType(text)) {
        "stremio_manifest" -> AddonStoreInputType.STREMIO_MANIFEST
        "cloudstream_repo" -> AddonStoreInputType.CLOUDSTREAM_REPO
        "search_query" -> AddonStoreInputType.SEARCH_QUERY
        else -> AddonStoreInputType.UNKNOWN
    }

    override fun observeAddonStore(): Flow<AddonStoreUiState> = combine(
        pluginManager.installedPlugins,
        pluginManager.repositories,
        installedUserAddons,
        extras
    ) { installedPlugins, repos, userAddonsAndLoaded, ex ->
        buildState(installedPlugins, repos, userAddonsAndLoaded.first, userAddonsAndLoaded.second, ex)
    }

    private fun buildState(
        installedPlugins: List<InstalledPlugin>,
        repos: List<PluginRepositoryEntry>,
        userAddons: List<AddonDescriptor>,
        loaded: Boolean,
        ex: Extras
    ): AddonStoreUiState {
        val profile = activeProfile()
        val normalizedInstalledUrls = profile?.safeInstalledLocalAddons.orEmpty().map(::normalizeAddonUrlForProfile)
        val normalizedIdentities = normalizedInstalledUrls.map(::addonUrlIdentity)
        val disabledIdentities = profile?.disabledLocalAddons.orEmpty().map(::addonUrlIdentity).toSet()

        val fromRepository = userAddons.map { addon ->
            val normalizedUrl = normalizeAddonUrlForProfile(addon.transportUrl)
            InstalledAddonUiModel(
                name = addon.manifest.name.takeIf { it.isNotBlank() } ?: addonNameFromUrl(normalizedUrl),
                description = addon.manifest.description?.takeIf { it.isNotBlank() }.orEmpty(),
                url = normalizedUrl,
                logoUrl = addon.manifest.logo,
                version = addon.manifest.version,
                configUrl = addonConfigUrl(normalizedUrl),
                configurable = addon.manifest.configurable == true
            )
        }
        val repoIdentities = fromRepository.map { addonUrlIdentity(it.url) }.toSet()
        val localFallback = if (loaded) {
            profile?.safeInstalledLocalAddons.orEmpty()
                .filterNot { addonUrlIdentity(it) in repoIdentities }
                .map { url ->
                    val normalizedUrl = normalizeAddonUrlForProfile(url)
                    InstalledAddonUiModel(
                        name = addonNameFromUrl(normalizedUrl),
                        description = "",
                        url = normalizedUrl,
                        configUrl = addonConfigUrl(normalizedUrl)
                    )
                }
        } else {
            emptyList()
        }

        val merged = (fromRepository + localFallback)
            .distinctBy { addonUrlIdentity(it.url) }
            .sortedWith(
                compareBy<InstalledAddonUiModel> {
                    normalizedIdentities.indexOf(addonUrlIdentity(it.url)).takeIf { index -> index >= 0 } ?: Int.MAX_VALUE
                }.thenBy { it.name.lowercase() }
            )
            .map { model ->
                val identity = addonUrlIdentity(model.url)
                val localIndex = normalizedIdentities.indexOf(identity)
                val canRemove = localIndex >= 0
                model.copy(
                    isEnabled = identity !in disabledIdentities,
                    canRemove = canRemove,
                    canMoveUp = canRemove && localIndex > 0,
                    canMoveDown = canRemove && localIndex < normalizedInstalledUrls.lastIndex,
                    isRefreshing = ex.refreshingUrl != null && addonUrlIdentity(ex.refreshingUrl) == identity
                )
            }

        val pluginModels = ex.openRepoPlugins.map { plugin ->
            val key = pluginManager.pluginInstallKey(ex.openRepoUrl, plugin.internalName)
            CloudstreamPluginUiModel(
                internalName = plugin.internalName,
                name = plugin.name,
                description = plugin.description,
                iconUrl = plugin.iconUrl,
                typesLabel = plugin.getDisplayTypes(),
                isInstalled = installedPlugins.any { pluginManager.pluginInstallKey(it.repositoryUrl, it.internalName) == key }
            )
        }

        return AddonStoreUiState(
            installedAddons = merged,
            cloudstreamRepos = repos.map { CloudstreamRepoUiModel(name = it.name, url = it.url, iconUrl = it.iconUrl) },
            accentColorArgb = profile?.safeAccentColorArgb?.toLong()?.and(0xffffffffL) ?: 0xFF4CAF50L,
            isLoading = profile != null && !loaded,
            inputText = ex.inputText,
            inputDetectedType = ex.detectedType,
            isSubmittingInput = ex.isSubmittingInput,
            inputError = ex.inputError,
            openRepoUrl = ex.openRepoUrl,
            openRepoName = ex.openRepoName,
            openRepoPlugins = pluginModels,
            isLoadingRepoPlugins = ex.isLoadingRepoPlugins,
            installingPluginKeys = ex.installingPluginKeys,
            repoDialogError = ex.repoDialogError
        )
    }

    override suspend fun refresh() {
        val profile = activeProfile()
        if (profile == null) {
            installedUserAddons.value = emptyList<AddonDescriptor>() to true
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
                installLocalAddonForProfile(activeProfile(), trimmed, profileManager, homeViewModel, onProfileChanged)
                refreshProfileState()
                extras.update { it.copy(inputText = "", detectedType = AddonStoreInputType.UNKNOWN, inputError = null) }
            }
            AddonStoreInputType.CLOUDSTREAM_REPO -> {
                extras.update { it.copy(isSubmittingInput = true, inputError = null) }
                val normalizedUrl = FluxaCoreNative.normalizeCloudstreamRepoUrl(trimmed)
                val result = pluginManager.addRepository(normalizedUrl)
                extras.update {
                    if (result.isSuccess) {
                        it.copy(inputText = "", isSubmittingInput = false, detectedType = AddonStoreInputType.UNKNOWN)
                    } else {
                        it.copy(
                            isSubmittingInput = false,
                            inputError = result.exceptionOrNull()?.message
                                ?: AppStrings.t(activeProfile()?.language ?: "en", "addons.repository_add_failed")
                        )
                    }
                }
            }
            else -> Unit
        }
    }

    override suspend fun toggleAddon(url: String, enabled: Boolean) {
        setLocalAddonEnabledForProfile(activeProfile(), url, enabled, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    override suspend fun removeAddon(url: String) {
        removeLocalAddonForProfile(activeProfile(), url, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    override suspend fun moveAddon(url: String, direction: Int) {
        moveLocalAddonForProfile(activeProfile(), url, direction, profileManager, homeViewModel, onProfileChanged)
        refreshProfileState()
    }

    private fun refreshProfileState() {
        extras.update { it.copy(profileRevision = it.profileRevision + 1) }
    }

    override suspend fun refreshAddon(url: String) {
        extras.update { it.copy(refreshingUrl = url) }
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

    override suspend fun openRepo(url: String) {
        val repoName = pluginManager.repositories.value.find { it.url == url }?.name?.takeIf { it.isNotBlank() }
        extras.update {
            it.copy(openRepoUrl = url, openRepoName = repoName, isLoadingRepoPlugins = true, openRepoPlugins = emptyList(), repoDialogError = null)
        }
        val plugins = pluginManager.getPluginsFromRepository(url)
        extras.update { it.copy(openRepoPlugins = plugins, isLoadingRepoPlugins = false) }
    }

    override suspend fun dismissRepoDialog() {
        extras.update { it.copy(openRepoUrl = null, openRepoName = null, openRepoPlugins = emptyList(), repoDialogError = null) }
    }

    override suspend fun removeRepo(url: String) {
        pluginManager.removeRepository(url)
    }

    override suspend fun toggleRepoPlugin(repoUrl: String, internalName: String) {
        val key = pluginManager.pluginInstallKey(repoUrl, internalName)
        val plugin = extras.value.openRepoPlugins.find { it.internalName == internalName } ?: return
        val isInstalled = pluginManager.installedPlugins.value.any {
            pluginManager.pluginInstallKey(it.repositoryUrl, it.internalName) == key
        }
        extras.update { it.copy(installingPluginKeys = it.installingPluginKeys + key, repoDialogError = null) }
        if (isInstalled) {
            pluginManager.uninstallPlugin(repoUrl, internalName)
        } else {
            val result = pluginManager.installPlugin(plugin, repoUrl)
            if (result.isFailure) {
                extras.update {
                    it.copy(repoDialogError = result.exceptionOrNull()?.message ?: AppStrings.t(activeProfile()?.language ?: "en", "auto.install_failed"))
                }
            }
        }
        extras.update { it.copy(installingPluginKeys = it.installingPluginKeys - key) }
    }
}
