package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.TraktIntegration

object ContinueWatchingListMerger {
    fun identityKey(meta: Meta): String {
        return TraktIntegration.contentIdentityKey(meta)
    }
}
