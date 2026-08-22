package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta

data class SearchResultRow(
    val title: String,
    val items: List<Meta>,
    val id: String,
    val type: String,
    val sourceAddonTransportUrl: String? = null,
    val sourceAddonCatalogType: String? = null
)
