package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder

fun LibraryUserCollectionFolder.effectiveImageUrl(): String? =
    FluxaCoreNative.collectionFolderPresentation(this).imageUrl

fun LibraryUserCollectionFolder.effectiveShape(): String =
    FluxaCoreNative.collectionFolderPresentation(this).shape

fun LibraryUserCollection.effectiveFolderShape(folder: LibraryUserCollectionFolder): String? =
    FluxaCoreNative.collectionFolderPresentation(folder).shape
