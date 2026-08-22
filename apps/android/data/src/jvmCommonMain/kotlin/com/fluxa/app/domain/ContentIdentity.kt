package com.fluxa.app.domain

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.core.rust.FluxaCoreNative

object ContentIdentity {
    fun traktKey(meta: Meta): String {
        return FluxaCoreNative.contentTraktKey(meta)
    }

    fun traktKeysBatch(metas: List<Meta>): List<String> {
        return FluxaCoreNative.contentTraktKeysBatch(metas)
    }

    fun mergeKeys(meta: Meta): Set<String> {
        return FluxaCoreNative.contentMergeKeys(meta)
    }

    fun watchedKeysBatch(metas: List<Meta>): List<Set<String>> {
        return FluxaCoreNative.contentWatchedKeysBatch(metas)
    }
}
