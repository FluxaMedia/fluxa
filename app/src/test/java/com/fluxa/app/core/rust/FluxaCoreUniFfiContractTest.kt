package com.fluxa.app.core.rust

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class FluxaCoreUniFfiContractTest {
    @Test
    fun uniffiHeadlessEngineDrivesSameJsonContractAsJni() {
        assertTrue(FluxaCoreUniFfi.version().isNotBlank())

        FluxaCoreUniFfi.createHeadlessEngine().use { engine ->
            val requested = engine.dispatch(
                mapOf(
                    "type" to "detailLoadRequested",
                    "contentType" to "movie",
                    "id" to "tt1",
                    "language" to "en"
                )
            )

            assertEquals("fetchMetaDetail", requested.effects[0].type)
            assertEquals("readPlaybackProgress", requested.effects[1].type)

            val completed = engine.completeEffect(
                mapOf(
                    "effectId" to requested.effects[0].id,
                    "status" to "ok",
                    "value" to mapOf("id" to "tt1", "name" to "Movie")
                )
            )
            val detail = completed.state["detail"] as Map<*, *>
            val meta = detail["meta"] as Map<*, *>
            assertEquals("Movie", meta["name"])
        }
    }
}
