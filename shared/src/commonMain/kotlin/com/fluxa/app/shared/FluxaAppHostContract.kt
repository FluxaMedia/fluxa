package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.settings.SettingsDataSource
import com.fluxa.app.shared.feature.streambadges.StreamBadgesDataSource
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.detail.DetailNavigationEvent
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.localmedia.LocalMediaKind
import com.fluxa.app.shared.feature.localmedia.LocalMediaPickedFolder
import com.fluxa.app.ui.catalog.DeviceType

data class FluxaAppDataSources(
    val catalogHome: CatalogHomeDataSource,
    val detail: DetailDataSource? = null,
    val calendar: CalendarDataSource? = null,
    val discover: DiscoverDataSource? = null,
    val library: LibraryDataSource? = null,
    val search: SearchDataSource? = null,
    val profile: ProfileDataSource? = null,
    val addonStore: AddonStoreDataSource? = null,
    val plugins: PluginsDataSource? = null,
    val auth: AuthDataSource? = null,
    val settings: SettingsDataSource? = null,
    val streamBadges: StreamBadgesDataSource? = null,
)

/** Stable platform-facing configuration for [FluxaAppHost]. */
data class FluxaAppHostConfig(
    val deviceType: DeviceType = DeviceType.Mobile,
    val isLowRamDevice: Boolean = false,
    val language: String? = null,
    val destination: FluxaDestination? = null,
    val detailRequest: DetailRequestUiModel? = null,
    val showNavigationBar: Boolean = true,
    val authStartOnNuvio: Boolean = false,
    val biometricAvailable: Boolean = false,
    val settingsPopRequestId: Int = 0,
    val overlayPopRequestId: Int = 0,
)

data class FluxaAppHostVisuals(
    val nuvioIcon: @Composable () -> Unit = {},
    val stremioIcon: @Composable () -> Unit = {},
    val authBackdrop: (@Composable () -> Unit)? = null,
    val traktIcon: @Composable () -> Unit = {},
    val simklIcon: @Composable () -> Unit = {},
    val anilistIcon: @Composable () -> Unit = {},
)

data class FluxaAppNavigationCallbacks(
    val onCatalogAction: (CatalogAction) -> Unit = {},
    val onDetailNavigationEvent: (DetailNavigationEvent) -> Unit = {},
    val onDetailBackRequested: () -> Unit = {},
    val onOpenUrlRequested: (String) -> Unit = {},
    val onAddonStoreBackRequested: () -> Unit = {},
    val onPluginsBackRequested: () -> Unit = {},
    val onStreamBadgesBackRequested: () -> Unit = {},
    val onDownloadOpened: (String) -> Unit = {},
    val onDestinationChanged: (FluxaDestination) -> Unit = {},
)

data class FluxaAppAuthCallbacks(
    val onAuthBackRequested: () -> Unit = {},
    val onAuthCompleted: () -> Unit = {},
    val onConnectStremioRequested: () -> Unit = {},
    val onConnectNuvioRequested: () -> Unit = {},
    val onConnectStremioWithCredentials: (String, String) -> Unit = { _, _ -> },
    val onConnectNuvioWithCredentials: (String, String) -> Unit = { _, _ -> },
    val onConnectTraktRequested: () -> Unit = {},
    val onConnectSimklRequested: () -> Unit = {},
    val onSyncProviderRequested: (String) -> Unit = {},
    val onConnectAnilistRequested: () -> Unit = {},
)

data class FluxaAppProfileCallbacks(
    val onPickAvatarRequested: (onPicked: (String?) -> Unit) -> Unit = {},
    val onBiometricAuthRequested: (ProfileUiModel, onResult: (Boolean) -> Unit) -> Unit = { _, _ -> },
    val onProfileSelectionCompleted: (String) -> Unit = {},
)

data class FluxaAppSettingsCallbacks(
    val onManageAddonsRequested: () -> Unit = {},
    val onManagePluginsRequested: () -> Unit = {},
    val onManageStreamBadgesRequested: () -> Unit = {},
    val onCheckForUpdateRequested: () -> Unit = {},
    val onSettingsBackRequested: () -> Unit = {},
    val onSettingsCanPopChanged: (Boolean) -> Unit = {},
)

data class FluxaAppLibraryCallbacks(
    val onPickLocalMediaFolderRequested: (LocalMediaKind, (LocalMediaPickedFolder?) -> Unit) -> Unit = { _, onPicked -> onPicked(null) },
)

data class FluxaAppOverlayCallbacks(
    val onOverlayOpenChanged: (Boolean) -> Unit = {},
)

data class FluxaAppHostCallbacks(
    val navigation: FluxaAppNavigationCallbacks = FluxaAppNavigationCallbacks(),
    val auth: FluxaAppAuthCallbacks = FluxaAppAuthCallbacks(),
    val profile: FluxaAppProfileCallbacks = FluxaAppProfileCallbacks(),
    val settings: FluxaAppSettingsCallbacks = FluxaAppSettingsCallbacks(),
    val library: FluxaAppLibraryCallbacks = FluxaAppLibraryCallbacks(),
    val overlay: FluxaAppOverlayCallbacks = FluxaAppOverlayCallbacks(),
)
