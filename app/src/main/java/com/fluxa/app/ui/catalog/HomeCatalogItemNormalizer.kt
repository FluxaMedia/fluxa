package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta

internal object HomeCatalogItemNormalizer {
    fun normalize(
        items: List<Meta>,
        catalogId: String,
        @Suppress("UNUSED_PARAMETER") lang: String,
        genre: String? = null
    ): List<Meta> {
        return FluxaCoreNative.normalizeHomeCatalogItems(items, catalogId, genre)
    }
}
