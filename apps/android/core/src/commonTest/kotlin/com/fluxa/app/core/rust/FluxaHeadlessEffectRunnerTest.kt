package com.fluxa.app.core.rust

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FluxaHeadlessEffectRunnerTest {
    @Test
    fun drainsEffectsInCompletionOrder() = runTest {
        val completed = mutableListOf<String>()
        val engine = QueueEngine(
            listOf(
                NativeHeadlessEffect(id = "first", type = "load"),
                NativeHeadlessEffect(id = "second", type = "save")
            )
        )
        val runner = FluxaHeadlessEffectRunner(
            engine = engine,
            environment = object : HeadlessPlatformEnvironment {
                override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
                    completed += effect.id
                    return HeadlessEffectCompletion(effect.id, "ok")
                }
            }
        )

        val result = runner.dispatchAndDrain("start")

        assertEquals(listOf("first", "second"), completed)
        assertEquals("done", result.state["status"])
    }

    @Test
    fun routesEffectsThroughPortableHandlers() = runTest {
        val environment = HeadlessEffectEnvironment(
            mapOf("load" to HeadlessEffectHandler { effect -> effect.payload["value"] })
        )

        val completion = environment.execute(
            NativeHeadlessEffect(id = "effect", type = "load", payload = mapOf("value" to 42))
        )

        assertEquals("ok", completion.status)
        assertEquals(42, completion.value)
    }

    @Test
    fun mergesStatePatchesWhileDraining() = runTest {
        var completed = false
        val engine = object : FluxaHeadlessEngine {
            override fun dispatch(action: Any) = NativeHeadlessEngineResult(
                effects = listOf(NativeHeadlessEffect("effect", "load")),
                stateProvider = { mapOf("home" to "ready") }
            )

            override fun completeEffect(result: Any): NativeHeadlessEngineResult {
                completed = true
                return NativeHeadlessEngineResult(stateProvider = { mapOf("profile" to "active") })
            }
        }
        val runner = FluxaHeadlessEffectRunner(
            engine,
            HeadlessEffectEnvironment(mapOf("load" to HeadlessEffectHandler { null }))
        )

        val result = runner.dispatchAndDrain("start")

        assertEquals(true, completed)
        assertEquals(mapOf("home" to "ready", "profile" to "active"), result.state)
    }
}

private class QueueEngine(
    effects: List<NativeHeadlessEffect>
) : FluxaHeadlessEngine {
    private val pending = effects.toMutableList()

    override fun dispatch(action: Any): NativeHeadlessEngineResult = next()

    override fun completeEffect(result: Any): NativeHeadlessEngineResult {
        pending.removeFirstOrNull()
        return next()
    }

    private fun next(): NativeHeadlessEngineResult {
        return NativeHeadlessEngineResult(
            effects = pending.take(1),
            stateProvider = { mapOf("status" to if (pending.isEmpty()) "done" else "pending") }
        )
    }
}
