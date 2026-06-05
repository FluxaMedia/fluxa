package com.fluxa.app.core.rust

import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class FluxaHeadlessEffectRunnerTest {
    @Test
    fun runnerExecutesEffectsAndFeedsResultsBackToCore() = runBlocking {
        FluxaCoreNative.createHeadlessEngine().use { engine ->
            val executedEffects = mutableListOf<String>()
            val runner = FluxaHeadlessEffectRunner(
                engine = engine,
                environment = object : HeadlessPlatformEnvironment {
                    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
                        executedEffects += effect.type
                        return when (effect.type) {
                            "fetchMetaDetail" -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "ok",
                                value = mapOf("id" to "tt1", "name" to "Movie")
                            )
                            "readPlaybackProgress" -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "ok",
                                value = null
                            )
                            else -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "error",
                                error = mapOf("code" to "unsupported_effect")
                            )
                        }
                    }
                }
            )

            val result = runner.dispatchAndDrain(
                mapOf(
                    "type" to "detailLoadRequested",
                    "contentType" to "movie",
                    "id" to "tt1",
                    "language" to "en"
                )
            )

            val detail = result.state["detail"] as Map<*, *>
            val meta = detail["meta"] as Map<*, *>
            assertEquals(false, detail["isLoading"])
            assertEquals("Movie", meta["name"])
            assertEquals(listOf("fetchMetaDetail", "readPlaybackProgress"), executedEffects)
        }
    }

    @Test
    fun runtimePublishesRustStateAfterDrainingEffects() = runBlocking {
        FluxaCoreNative.createHeadlessEngine().use { engine ->
            val runtime = FluxaHeadlessAppRuntime(
                engine = engine,
                environment = object : HeadlessPlatformEnvironment {
                    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
                        return when (effect.type) {
                            "fetchMetaDetail" -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "ok",
                                value = mapOf("id" to "tt1", "name" to "Movie")
                            )
                            "readPlaybackProgress" -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "ok",
                                value = null
                            )
                            else -> HeadlessEffectCompletion(
                                effectId = effect.id,
                                status = "error",
                                error = mapOf("code" to "unsupported_effect")
                            )
                        }
                    }
                }
            )

            val result = runtime.dispatch(
                mapOf(
                    "type" to "detailLoadRequested",
                    "contentType" to "movie",
                    "id" to "tt1",
                    "language" to "en"
                )
            )

            val detail = runtime.state.value["detail"] as Map<*, *>
            val meta = detail["meta"] as Map<*, *>
            assertEquals(result.state, runtime.state.value)
            assertEquals(false, detail["isLoading"])
            assertEquals("Movie", meta["name"])
        }
    }
}
