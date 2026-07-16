package com.fluxa.app.core.rust

class NativeHeadlessEngineResult(
    val effects: List<NativeHeadlessEffect> = emptyList(),
    stateProvider: () -> Map<String, Any?> = { emptyMap() }
) {
    val state: Map<String, Any?> by lazy(stateProvider)
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
    suspend fun dispatchAndDrain(action: Any): NativeHeadlessEngineResult = drain(engine.dispatch(action))

    suspend fun completeAndDrain(result: HeadlessEffectCompletion): NativeHeadlessEngineResult {
        return drain(engine.completeEffect(result))
    }

    private suspend fun drain(initial: NativeHeadlessEngineResult): NativeHeadlessEngineResult {
        var current = initial
        val patches = mutableListOf(initial)
        var remaining = maxEffectsPerDispatch
        while (current.effects.isNotEmpty() && remaining > 0) {
            current = engine.completeEffect(environment.execute(current.effects.first()))
            patches += current
            remaining--
        }
        return NativeHeadlessEngineResult(
            effects = current.effects,
            stateProvider = { patches.fold(emptyMap<String, Any?>()) { state, patch -> state + patch.state } }
        )
    }
}
