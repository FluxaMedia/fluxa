package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.core.rust.FluxaCoreNative

object ContinueWatchingListMerger {
    fun mergeDuplicates(items: List<Meta>): List<Meta> {
        return FluxaCoreNative.mergeContinueWatchingDuplicates(items)
    }

    fun identityKey(meta: Meta): String {
        return TraktIntegration.contentIdentityKey(meta)
    }
}
