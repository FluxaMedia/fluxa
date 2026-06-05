package com.fluxa.app.core.rust.models

data class NativeSearchResultGrouping(
    val groups: List<Map<String, Any?>> = emptyList(),
    val totalCount: Int = 0,
    val query: String = ""
)
