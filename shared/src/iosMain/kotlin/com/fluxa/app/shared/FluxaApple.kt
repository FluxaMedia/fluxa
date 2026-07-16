package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.shared.platform.AppleCatalogHomeSnapshot
import com.fluxa.app.shared.platform.AppleCatalogItemSnapshot
import com.fluxa.app.shared.platform.AppleSearchSnapshot
import com.fluxa.app.shared.platform.AppleDiscoverRequestSnapshot
import com.fluxa.app.shared.platform.AppleDiscoverSnapshot
import com.fluxa.app.shared.platform.AppleCalendarSnapshot
import com.fluxa.app.shared.platform.AppleAuthSubmitSnapshot
import com.fluxa.app.shared.platform.AppleAuthSnapshot
import com.fluxa.app.shared.platform.AppleLibrarySnapshot
import com.fluxa.app.shared.platform.AppleDetailRequestSnapshot
import com.fluxa.app.shared.platform.AppleDetailSnapshot
import com.fluxa.app.shared.platform.AppleDetailStreamSnapshot
import com.fluxa.app.shared.platform.ApplePlaybackRequestSnapshot
import com.fluxa.app.shared.platform.AppleAddonStoreDataSource
import com.fluxa.app.shared.platform.AppleAuthDataSource
import com.fluxa.app.shared.platform.AppleCatalogHomeDataSource
import com.fluxa.app.shared.platform.AppleCalendarDataSource
import com.fluxa.app.shared.platform.AppleDetailDataSource
import com.fluxa.app.shared.platform.AppleDiscoverDataSource
import com.fluxa.app.shared.platform.AppleFluxaPlatformServices
import com.fluxa.app.shared.platform.AppleLibraryDataSource
import com.fluxa.app.shared.platform.AppleProfileDataSource
import com.fluxa.app.shared.platform.AppleSearchDataSource
import com.fluxa.app.shared.platform.AppleSettingsDataSource
import com.fluxa.app.data.apple.AppleAddonManifestSnapshot
import com.fluxa.app.data.apple.AppleStremioBridge
import com.fluxa.app.data.local.AppleWatchlistStore
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import platform.UIKit.UIViewController

object FluxaApple {
    private val json = Json
    internal val watchlistStore = AppleWatchlistStore()
    internal val catalogHomeDataSource = AppleCatalogHomeDataSource()
    internal val detailDataSource = AppleDetailDataSource(watchlistStore)
    internal val searchDataSource = AppleSearchDataSource()
    internal val discoverDataSource = AppleDiscoverDataSource()
    internal val calendarDataSource = AppleCalendarDataSource()
    internal val libraryDataSource = AppleLibraryDataSource(watchlistStore)
    internal val profileDataSource = AppleProfileDataSource(watchlistStore::setActiveProfile)
    internal val settingsDataSource = AppleSettingsDataSource()
    internal val addonStoreDataSource = AppleAddonStoreDataSource()
    internal val authDataSource = AppleAuthDataSource()
    internal val platformServices = AppleFluxaPlatformServices(
        catalogHomeDataSource,
        detailDataSource,
        searchDataSource,
        discoverDataSource,
        calendarDataSource,
        libraryDataSource,
        profileDataSource,
        settingsDataSource,
        addonStoreDataSource,
        authDataSource
    )
    private var onPlaybackRequested: (ApplePlaybackRequestSnapshot) -> Unit = {}

    fun rootViewController(): UIViewController = ComposeUIViewController {
        FluxaAppleApp()
    }

    fun setCatalogHomeRefreshHandler(handler: () -> Unit) {
        catalogHomeDataSource.setOnRefreshRequested(handler)
    }

    fun updateCatalogHome(snapshot: AppleCatalogHomeSnapshot) {
        catalogHomeDataSource.update(snapshot)
    }

    fun setSearchHandler(handler: (String) -> Unit) {
        searchDataSource.setOnSearchRequested(handler)
    }

    fun updateSearch(snapshot: AppleSearchSnapshot) {
        searchDataSource.update(snapshot)
    }

    fun setDiscoverHandler(handler: (AppleDiscoverRequestSnapshot) -> Unit) {
        discoverDataSource.setOnDiscoverRequested(handler)
    }

    fun updateDiscover(snapshot: AppleDiscoverSnapshot) {
        discoverDataSource.update(snapshot)
    }

    fun setCalendarMonthHandler(handler: (Int, Int) -> Unit) {
        calendarDataSource.setOnMonthRequested(handler)
    }

    fun updateCalendar(snapshot: AppleCalendarSnapshot) {
        calendarDataSource.update(snapshot)
    }

    fun setAuthSubmitHandler(handler: (AppleAuthSubmitSnapshot) -> Unit) {
        authDataSource.setOnSubmitRequested(handler)
    }

    fun updateAuth(snapshot: AppleAuthSnapshot) {
        authDataSource.update(snapshot)
    }

    fun setLibraryRefreshHandler(handler: () -> Unit) {
        libraryDataSource.setOnRefreshRequested(handler)
    }

    fun updateLibrary(snapshot: AppleLibrarySnapshot) {
        libraryDataSource.update(snapshot)
    }

    fun parseAddonManifest(body: String): AppleAddonManifestSnapshot? = AppleStremioBridge.parseManifest(body)

    fun normalizeAddonManifestUrl(rawUrl: String): String = AppleStremioBridge.normalizeManifestUrl(rawUrl)

    fun addonResourceUrl(transportUrl: String, resource: String, contentType: String, id: String): String =
        AppleStremioBridge.resourceUrl(transportUrl, resource, contentType, id)

    fun addonCatalogUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        extraName: String?,
        extraValue: String?
    ): String = AppleStremioBridge.catalogUrl(transportUrl, contentType, catalogId, extraName, extraValue)

    fun parseCatalogItems(
        body: String,
        fallbackType: String,
        addonTransportUrl: String?,
        catalogType: String?
    ): List<AppleCatalogItemSnapshot>? = AppleStremioBridge.parseCatalogItems(body, fallbackType)?.map { item ->
        AppleCatalogItemSnapshot(
            id = item.id,
            type = item.type,
            title = item.title,
            subtitle = item.subtitle,
            artworkUrl = item.artworkUrl,
            logoUrl = item.logoUrl,
            addonTransportUrl = addonTransportUrl,
            catalogType = catalogType
        )
    }

    fun parseDirectStreams(body: String, addonName: String): List<AppleDetailStreamSnapshot>? =
        AppleStremioBridge.parseDirectStreams(body)?.mapNotNull { stream ->
            val playableUrl = stream.playableUrl ?: return@mapNotNull null
            AppleDetailStreamSnapshot(
                addonName = addonName,
                title = stream.title ?: addonName,
                playableUrl = playableUrl,
                requestHeadersJson = json.encodeToString(stream.requestHeaders)
            )
        }

    fun setDetailHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit
    ) {
        detailDataSource.setHandlers(load, watchlist)
    }

    fun updateDetail(snapshot: AppleDetailSnapshot) {
        detailDataSource.update(snapshot)
    }

    fun setPlaybackHandler(handler: (ApplePlaybackRequestSnapshot) -> Unit) {
        onPlaybackRequested = handler
    }

    internal fun requestPlayback(snapshot: ApplePlaybackRequestSnapshot) {
        onPlaybackRequested(snapshot)
    }

    internal fun requestFirstAvailablePlayback(resumePositionMs: Long) {
        detailDataSource.firstPlaybackRequest(resumePositionMs)?.let(onPlaybackRequested)
    }






}

@Composable
private fun FluxaAppleApp() {
    FluxaAppHost(
        platformServices = FluxaApple.platformServices,
        onDetailNavigationEvent = { event ->
            if (event is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.PlayStream) {
                FluxaApple.requestPlayback(
                    ApplePlaybackRequestSnapshot(
                        playableUrl = event.stream.playableUrl,
                        title = event.stream.title,
                        resumePositionMs = event.resumeProgress,
                        requestHeadersJson = event.stream.requestHeadersJson
                    )
                )
            } else if (event is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.SelectSources) {
                FluxaApple.requestFirstAvailablePlayback(event.resumeProgress)
            }
        }
    )
}
