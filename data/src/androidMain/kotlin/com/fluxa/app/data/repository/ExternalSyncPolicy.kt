package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video

object ExternalSyncPolicy {
    fun afterResponse(provider: ExternalSyncProvider, statusCode: Int): ExternalSyncAction =
        FluxaCoreNative.externalSyncResponseAction(provider.name.lowercase(), statusCode).toAction()

    fun afterRefreshRetry(statusCode: Int?): ExternalSyncAction =
        FluxaCoreNative.externalSyncRefreshRetryAction(statusCode).toAction()

    fun malWatchedUpdate(meta: Meta, episodes: List<Video>): MalListUpdate? =
        FluxaCoreNative.malListUpdate(meta, episodes, watched = true)

    fun malWatchlistUpdate(meta: Meta): MalListUpdate? =
        FluxaCoreNative.malListUpdate(meta, emptyList(), watched = false)
}

private fun String.toAction(): ExternalSyncAction = when (this) {
    "stamp_success" -> ExternalSyncAction.STAMP_SUCCESS
    "clear_credentials" -> ExternalSyncAction.CLEAR_CREDENTIALS
    "refresh_credentials" -> ExternalSyncAction.REFRESH_CREDENTIALS
    else -> ExternalSyncAction.KEEP_CREDENTIALS
}
