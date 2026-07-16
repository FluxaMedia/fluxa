package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.TraktSyncItem

internal fun TraktSyncItem.toMeta(type: String, unknownName: () -> String) =
    TraktSyncMapper.toMeta(this, type, unknownName, TraktIntegration::contentIdFrom)

internal suspend fun fetchTraktSyncPages(
    request: suspend (page: Int, limit: Int) -> retrofit2.Response<List<TraktSyncItem>>
): List<TraktSyncItem> {
    val limit = 1000
    val first = request(1, limit)
    if (!first.isSuccessful) return emptyList()
    val results = first.body().orEmpty().toMutableList()
    val pageCount = (first.headers()["X-Pagination-Page-Count"]?.toIntOrNull() ?: 1).coerceAtLeast(1)
    val cappedPageCount = pageCount.coerceAtMost(20)
    for (page in 2..cappedPageCount) {
        val response = request(page, limit)
        if (!response.isSuccessful) break
        results += response.body().orEmpty()
    }
    return results
}
