package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import kotlin.test.Test
import kotlin.test.assertEquals

class ExternalSyncPolicyTest {
    @Test
    fun delegatesCredentialAndMalUpdateDecisionsToCore() {
        assertEquals(
            ExternalSyncAction.REFRESH_CREDENTIALS,
            ExternalSyncPolicy.afterResponse(ExternalSyncProvider.MAL, 401)
        )
        assertEquals(
            ExternalSyncAction.CLEAR_CREDENTIALS,
            ExternalSyncPolicy.afterResponse(ExternalSyncProvider.SIMKL, 401)
        )
        assertEquals(
            "completed",
            ExternalSyncPolicy.malWatchedUpdate(
                Meta(id = "mal:42", name = "Anime", type = "series", episodesCount = 12),
                listOf(Video(id = "episode", number = 12))
            )?.status
        )
    }
}
