package com.fluxa.app.core.rust

import com.fluxa.app.core.rust.models.NativeHeadlessEffect

data class HeadlessEffectCompletion(
    val effectId: String,
    val status: String,
    val value: Any? = null,
    val error: Any? = null
)

interface HeadlessPlatformEnvironment {
    suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion
}

interface FluxaHeadlessEngine {
    fun dispatch(action: Any): NativeHeadlessEngineResult
    fun completeEffect(result: Any): NativeHeadlessEngineResult
}

class FluxaHeadlessEffectRunner(
    private val engine: FluxaHeadlessEngine,
    private val environment: HeadlessPlatformEnvironment,
    private val maxEffectsPerDispatch: Int = 64
) {
    suspend fun dispatchAndDrain(action: Any): NativeHeadlessEngineResult {
        return drain(engine.dispatch(action))
    }

    suspend fun completeAndDrain(result: HeadlessEffectCompletion): NativeHeadlessEngineResult {
        return drain(engine.completeEffect(result))
    }

    private suspend fun drain(initial: NativeHeadlessEngineResult): NativeHeadlessEngineResult {
        var current = initial
        var remaining = maxEffectsPerDispatch
        while (current.effects.isNotEmpty() && remaining > 0) {
            val completion = environment.execute(current.effects.first())
            current = engine.completeEffect(completion)
            remaining--
        }
        return current
    }
}
