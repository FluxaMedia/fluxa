package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.toCatalogCardUiModel
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.effectiveImageUrl

/** Canonical folder -> shared UI mapping used by Home, Android Library and Desktop Library. */
fun LibraryUserCollectionFolder.toLibraryFolderUiModel(): LibraryFolderUiModel = LibraryFolderUiModel(
    id = id,
    title = title,
    imageUrl = imageUrl,
    shape = shape,
    catalogTitle = catalogTitle,
    hideTitle = hideTitle == true,
    focusGifEnabled = focusGifEnabled != false,
    coverEmoji = coverEmoji,
    coverImageUrl = coverImageUrl,
    focusGifUrl = focusGifUrl,
    heroBackdropUrl = heroBackdropUrl,
)

fun LibraryUserCollectionFolder.toLibraryCatalogItem(
    deviceType: DeviceType,
    posterWidthPreset: String? = null,
): CatalogItemUiModel = CatalogItemUiModel(
    id = id,
    type = "catalog_folder",
    card = if (posterWidthPreset == null) {
        toLibraryFolderUiModel().toCatalogCardUiModel(deviceType = deviceType)
    } else {
        toLibraryFolderUiModel().toCatalogCardUiModel(posterWidthPreset, deviceType)
    },
    backdropUrl = heroBackdropUrl ?: effectiveImageUrl(),
)
