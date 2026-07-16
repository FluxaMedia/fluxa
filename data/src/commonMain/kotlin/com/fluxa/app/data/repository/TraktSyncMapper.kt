package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktIds
import com.fluxa.app.data.remote.TraktSyncItem

object TraktSyncMapper {
    fun toMeta(
        item: TraktSyncItem,
        type: String,
        unknownName: () -> String,
        resolveContentId: (TraktIds) -> String?
    ): Meta? {
        val summary = item.movie ?: item.show ?: return null
        val id = resolveContentId(summary.ids) ?: return null
        return Meta(
            id = id,
            name = summary.title ?: unknownName(),
            type = type,
            poster = null,
            releaseInfo = summary.year?.toString(),
            released = summary.year?.let { "$it-01-01" }
        )
    }
}
