package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioLibraryItem
import com.fluxa.app.data.remote.Video

object NuvioSyncRequests {
    fun collection(collection: LibraryUserCollection): Map<String, Any?> =
        FluxaCoreNative.nuvioRequest("nuvioCollectionRequest", collection)

    fun libraryItem(meta: Meta, addedAt: Long): Map<String, Any?> =
        FluxaCoreNative.nuvioRequest("nuvioLibraryItemRequest", mapOf("item" to meta, "addedAt" to addedAt))

    fun libraryItem(item: NuvioLibraryItem): Map<String, Any?> =
        FluxaCoreNative.nuvioRequest("nuvioLibraryItemRequest", mapOf("item" to item, "addedAt" to item.addedAt))

    fun watchedItems(meta: Meta, episodes: List<Video>, watchedAt: Long): List<Map<String, Any?>> =
        FluxaCoreNative.nuvioRequestList("nuvioWatchedItemsRequest", mapOf("meta" to meta, "episodes" to episodes, "watchedAt" to watchedAt))

    fun playbackProgress(meta: Meta, videoId: String?, position: Long, duration: Long, watchedAt: Long): Map<String, Any?> =
        FluxaCoreNative.nuvioRequest("nuvioPlaybackProgressRequest", mapOf("meta" to meta, "videoId" to videoId, "position" to position, "duration" to duration, "watchedAt" to watchedAt))
}
