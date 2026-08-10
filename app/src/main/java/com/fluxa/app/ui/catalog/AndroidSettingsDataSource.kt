package com.fluxa.app.ui.catalog

import android.content.Context
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.buildMetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.moveMetadataFeedOrder
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import com.fluxa.app.domain.discovery.toggleMetadataFeed
import com.fluxa.app.player.LastMediaDebugInfoStore
import com.fluxa.app.shared.feature.settings.*
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch


class AndroidSettingsDataSource(
    private val context: Context,
    private val homeViewModel: HomeViewModel,
    private val profileManager: ProfileManager,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository,
    private val appVersionLabel: String,
    private val language: () -> String
) : SettingsDataSource {

    private val profileState = MutableStateFlow(activeProfile())

    override fun observeAppearanceHome(): Flow<SettingsAppearanceHomeUiModel> = profileFlow()
        .map { profile -> profile?.toSettingsAppearanceHomeUiModel() ?: SettingsAppearanceHomeUiModel() }
        .distinctUntilChanged()

    override fun observeSettings(): Flow<SettingsUiState> = combine(
        profileFlow(),
        homeViewModel.userAddons,
        homeViewModel.loadedCs3CatalogFeedOptions,
        LastMediaDebugInfoStore.state,
        homeViewModel.syncingProviders,
        homeViewModel.connectErrors,
        homeViewModel.libraryUiState
    ) { values ->
        val profile = values[0] as UserProfile?
        @Suppress("UNCHECKED_CAST") val addons = values[1] as List<AddonDescriptor>
        @Suppress("UNCHECKED_CAST") val cs3Options = values[2] as List<MetadataFeedOption>
        val debugInfo = values[3] as com.fluxa.app.player.LastMediaDebugInfo
        @Suppress("UNCHECKED_CAST") val syncingProviders = values[4] as Set<String>
        @Suppress("UNCHECKED_CAST") val connectErrors = values[5] as Map<String, String>
        val libraryUiState = values[6] as LibraryUiState
        if (profile == null) {
            SettingsUiState(isLoading = true)
        } else {
            buildState(profile, addons.let { buildMetadataFeedOptions(it, language()) } + cs3Options, debugInfo, syncingProviders, connectErrors, libraryUiState)
        }
    }

    override suspend fun refreshContentFeeds() {
        homeViewModel.refreshInstalledAddons(forceRefresh = true)
    }

    private fun profileFlow(): Flow<UserProfile?> = callbackFlow {
        val listener: () -> Unit = { launch { profileState.value = activeProfile() } }
        profileManager.addChangeListener(listener)
        val forwardingJob = launch { profileState.collect { trySend(it) } }
        awaitClose {
            forwardingJob.cancel()
            profileManager.removeChangeListener(listener)
        }
    }

    private fun buildState(
        profile: UserProfile,
        metadataOptions: List<MetadataFeedOption>,
        debugInfo: com.fluxa.app.player.LastMediaDebugInfo,
        syncingProviders: Set<String> = emptySet(),
        connectErrors: Map<String, String> = emptyMap(),
        libraryUiState: LibraryUiState = LibraryUiState(),
    ): SettingsUiState = profile.toSettingsUiState(
        metadataOptions = metadataOptions,
        appVersionLabel = appVersionLabel,
        accountMetrics = providerMetrics(
            profile = profile,
            syncingProviders = syncingProviders,
            connectErrors = connectErrors,
        ),
        developerSnapshot = SettingsDeveloperSnapshot(
            lastProbeUpdatedAt = LastMediaDebugInfoStore.formattedUpdatedAt(debugInfo.updatedAtMs)
                .takeIf { debugInfo.hasInfo },
            lastProbeTitle = debugInfo.title.takeIf { debugInfo.hasInfo },
            lastProbeUrl = debugInfo.url.takeIf { debugInfo.hasInfo },
            technicalInfo = debugInfo.technicalInfo,
        ),
    ).let { state ->
        state.copy(
            playback = state.playback.copy(
                inAppPlayerOptions = listOf(
                    SettingsChoiceOption("internal", "ExoPlayer"),
                    SettingsChoiceOption("mpv", "libmpv"),
                ),
                externalPlayerOptions = AndroidExternalPlayerRegistry.installedVideoPlayers(context),
            )
        )
    }

    private fun providerMetrics(
        profile: UserProfile,
        syncingProviders: Set<String>,
        connectErrors: Map<String, String>,
    ): SettingsAccountMetrics {
        val stremio = thirdPartyProviderRepository.cached(profile, ThirdPartyProviderId.STREMIO)
        val nuvio = thirdPartyProviderRepository.cached(profile, ThirdPartyProviderId.NUVIO)
        val trakt = thirdPartyProviderRepository.cached(profile, ThirdPartyProviderId.TRAKT)
        val simkl = thirdPartyProviderRepository.cached(profile, ThirdPartyProviderId.SIMKL)
        val anilist = thirdPartyProviderRepository.cached(profile, ThirdPartyProviderId.ANILIST)
        return SettingsAccountMetrics(
            syncingProviders = syncingProviders,
            connectErrors = connectErrors,
            providerLastSyncAt = listOfNotNull(
                stremio?.syncedAt?.takeIf { it > 0L }?.let { ThirdPartyProviderId.STREMIO.key to it },
                nuvio?.syncedAt?.takeIf { it > 0L }?.let { ThirdPartyProviderId.NUVIO.key to it },
                trakt?.syncedAt?.takeIf { it > 0L }?.let { ThirdPartyProviderId.TRAKT.key to it },
                simkl?.syncedAt?.takeIf { it > 0L }?.let { ThirdPartyProviderId.SIMKL.key to it },
                anilist?.syncedAt?.takeIf { it > 0L }?.let { ThirdPartyProviderId.ANILIST.key to it },
            ).toMap(),
            continueWatchingCount = homeViewModel.currentContinueWatchingCount.value,
            stremioItemCount = stremio?.itemCount ?: 0,
            stremioContinueWatchingCount = stremio?.continueWatching?.size ?: 0,
            stremioLibraryCount = stremio?.libraryCount ?: 0,
            stremioAddonCount = stremio?.addonCount ?: 0,
            nuvioItemCount = nuvio?.itemCount ?: 0,
            nuvioContinueWatchingCount = nuvio?.continueWatching?.size ?: 0,
            nuvioLibraryCount = nuvio?.libraryCount ?: 0,
            nuvioAddonCount = nuvio?.addonCount ?: 0,
            traktItemCount = trakt?.itemCount,
            traktContinueWatchingCount = trakt?.continueWatching?.size,
            traktLibraryCount = trakt?.libraryCount,
            simklItemCount = simkl?.itemCount ?: 0,
            simklContinueWatchingCount = simkl?.continueWatching?.size ?: 0,
            simklLibraryCount = simkl?.libraryCount ?: 0,
            anilistItemCount = anilist?.itemCount ?: 0,
            anilistContinueWatchingCount = anilist?.continueWatching?.size ?: 0,
            anilistLibraryCount = anilist?.libraryCount ?: 0,
        )
    }

    private fun update(block: (UserProfile) -> UserProfile) {
        val profile = activeProfile() ?: return
        val updated = profileManager.updateProfile(profile.id, block) ?: return
        profileState.value = updated
        onProfileChanged(updated)
    }

    override suspend fun updateGeneral(value: SettingsGeneralUiModel) = update { it.withGeneralSettings(value) }

    override suspend fun updateAppearance(value: SettingsAppearanceUiModel) = update { it.withAppearanceSettings(value) }

    override suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel) = update { it.withAppearanceHomeSettings(value) }

    override suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel) = update { it.withAppearanceDetailSettings(value) }

    override suspend fun updatePlayback(value: SettingsPlaybackUiModel) = update { it.withPlaybackSettings(value) }

    override suspend fun updateSubtitles(value: SettingsSubtitlesUiModel) = update { it.withSubtitleSettings(value) }

    override suspend fun updateAdvanced(value: SettingsAdvancedUiModel) = update { it.withAdvancedSettings(value) }

    override suspend fun updateAddons(value: SettingsAddonsUiModel) = update { it.withAddonSettings(value) }

    override suspend fun updateDownloads(value: SettingsDownloadsUiModel) = update { it.withDownloadSettings(value) }

    override suspend fun updateSystem(value: SettingsSystemUiModel) = update { it.withSystemSettings(value) }

    override suspend fun updateTmdbAccount(value: SettingsAccountUiModel) = update { it.withAccountSettings(value) }

    override suspend fun updateNotifications(value: SettingsNotificationsUiModel) = update { it.withNotificationSettings(value) }

    override suspend fun updateShowHeroSection(value: Boolean) = update {
        it.copy(showHeroSection = value)
    }

    private fun currentMetadataOptions(): List<MetadataFeedOption> =
        buildMetadataFeedOptions(homeViewModel.userAddons.value, language()) + homeViewModel.loadedCs3CatalogFeedOptions.value

    override suspend fun toggleHeroFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val ordered = orderedMetadataFeeds(options, profile.heroFeedOrder).map { it.key }
        profile.copy(heroFeedToggles = toggleMetadataFeed(profile.heroFeedToggles, ordered, key, SETTINGS_HERO_FEED_LIMIT))
    }

    override suspend fun moveHeroFeed(key: String, direction: Int) = update { profile ->
        val options = currentMetadataOptions()
        profile.copy(heroFeedOrder = moveMetadataFeedOrder(options, profile.heroFeedOrder, key, direction))
    }

    override suspend fun toggleHomeFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val ordered = orderedMetadataFeeds(options, profile.homeFeedOrder).map { it.key }
        profile.copy(homeFeedToggles = toggleMetadataFeed(profile.homeFeedToggles, ordered, key))
    }

    override suspend fun moveHomeFeed(key: String, direction: Int) = update { profile ->
        val options = currentMetadataOptions()
        profile.copy(homeFeedOrder = moveMetadataFeedOrder(options, profile.homeFeedOrder, key, direction))
    }

    override suspend fun toggleTopTenFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val homeOrdered = orderedMetadataFeeds(options, profile.homeFeedOrder).map { it.key }
        val homeSelection = effectiveHomeMetadataFeedSelection(profile.homeFeedToggles, homeOrdered) ?: homeOrdered
        profile.copy(topTenFeedToggles = toggleMetadataFeed(profile.topTenFeedToggles, homeSelection, key))
    }

    override suspend fun disconnectSync() {
        val profile = activeProfile() ?: return
        thirdPartyProviderRepository.clearAll(profile)
        update { it.withAllSyncProvidersDisconnected() }
    }

    override suspend fun disconnectProvider(provider: String) {
        val profile = activeProfile() ?: return
        val providerId = ThirdPartyProviderId.from(provider) ?: return
        thirdPartyProviderRepository.clear(profile, providerId)
        update { it.withProviderDisconnected(providerId.key) }
    }
}
