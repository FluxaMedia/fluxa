package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeCardLayout
import com.fluxa.app.data.local.safePosterWidthPreset
import com.fluxa.app.ui.catalog.DeviceType

fun UserProfile?.toHomeCollectionRows(
    posterWidthPresetOverride: String? = null,
    deviceType: DeviceType = DeviceType.Mobile,
): List<CatalogRowUiModel> {
    val profile = this ?: return emptyList()
    val posterWidthPreset = posterWidthPresetOverride ?: profile.safePosterWidthPreset
    return profile.safeLibraryCollections
        .asSequence()
        .filter { it.showOnHome == true && it.folders.orEmpty().isNotEmpty() }
        .map { collection ->
            CatalogRowUiModel(
                id = "collection:${collection.id}",
                title = collection.title,
                categoryType = "catalog_folder",
                cardLayout = profile.safeCardLayout,
                items = collection.folders.orEmpty().map { folder ->
                    folder.toLibraryCatalogItem(
                        deviceType = deviceType,
                        posterWidthPreset = posterWidthPreset,
                    )
                },
            )
        }
        .toList()
}
