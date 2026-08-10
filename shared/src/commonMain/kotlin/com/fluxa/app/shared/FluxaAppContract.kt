package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthUiState
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.player.PlayerRenderAction
import com.fluxa.app.shared.feature.player.PlayerRenderState
import com.fluxa.app.shared.feature.plugins.PluginsAction
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsBrandIcons
import com.fluxa.app.shared.feature.settings.SettingsCategory
import com.fluxa.app.shared.feature.settings.SettingsUiState
import com.fluxa.app.ui.catalog.DeviceType

internal data class FluxaAppFeatureStates(
    val detail: DetailUiState? = null,
    val search: SearchUiState? = null,
    val discover: DiscoverUiState? = null,
    val calendar: CalendarUiState? = null,
    val library: LibraryUiState? = null,
    val profile: ProfileUiState? = null,
    val settings: SettingsUiState? = null,
    val addonStore: AddonStoreUiState? = null,
    val plugins: PluginsUiState? = null,
    val auth: AuthUiState? = null,
    val player: PlayerRenderState? = null,
)

internal data class FluxaAppActions(
    val onDestinationSelected: (FluxaDestination) -> Unit,
    val onCatalogAction: (CatalogAction) -> Unit,
    val onDetailAction: (DetailAction) -> Unit = {},
    val onDetailBackRequested: () -> Unit = {},
    val onSourceSelectionBackRequested: () -> Unit = {},
    val onCategoryBackRequested: () -> Unit = {},
    val onCategoryItemSelected: (CatalogItemUiModel) -> Unit = {},
    val onCategorySelected: (id: String, title: String) -> Unit = { _, _ -> },
    val onSearchAction: (SearchAction) -> Unit = {},
    val onDiscoverAction: (DiscoverAction) -> Unit = {},
    val onCalendarAction: (CalendarAction) -> Unit = {},
    val onNotificationsRequested: () -> Unit = {},
    val onNotificationsBackRequested: () -> Unit = {},
    val onLibraryItemSelected: (CatalogItemUiModel) -> Unit = {},
    val onLibraryAction: (LibraryAction) -> Unit = {},
    val onSettingsAction: (SettingsAction) -> Unit = {},
    val onSwitchProfilesRequested: () -> Unit = {},
    val onSettingsBackRequested: () -> Unit = {},
    val onSettingsPushCategory: (SettingsCategory) -> Unit = {},
    val onSettingsPopCategory: () -> Unit = {},
    val onSettingsSelectCategory: (SettingsCategory) -> Unit = {},
    val onAddonStoreAction: (AddonStoreAction) -> Unit = {},
    val onOpenUrlRequested: (String) -> Unit = {},
    val onAddonStoreBackRequested: () -> Unit = {},
    val onPluginsAction: (PluginsAction) -> Unit = {},
    val onPluginsBackRequested: () -> Unit = {},
    val onAuthAction: (AuthAction) -> Unit = {},
    val onProfileListAction: (ProfileAction) -> Unit = {},
    val onProfileBiometricRequested: (ProfileUiModel) -> Unit = {},
    val onPlayerAction: (PlayerRenderAction) -> Unit = {},
)

internal data class FluxaProfileEditorBindings(
    val avatarUrl: String? = null,
    val onPickAvatarClick: () -> Unit = {},
    val onRemoveAvatarClick: () -> Unit = {},
    val onPickPackAvatarClick: (String) -> Unit = {},
    val onProfileSave: (ProfileEditUiModel) -> Unit = {},
    val onProfileDelete: (suspend (String?) -> Boolean)? = null,
    val onProfileEditCancel: () -> Unit = {},
    val onPickBackgroundClick: () -> Unit = {},
)

internal data class FluxaAppPresentation(
    val deviceType: DeviceType = DeviceType.Mobile,
    val settingsBrandIcons: SettingsBrandIcons = SettingsBrandIcons(),
    val nuvioIcon: @Composable () -> Unit = {},
    val stremioIcon: @Composable () -> Unit = {},
    val authBackdrop: (@Composable () -> Unit)? = null,
    val biometricAvailable: Boolean = false,
    val showNavigationBar: Boolean = true,
)
