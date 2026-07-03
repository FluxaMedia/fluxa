package com.fluxa.app.core.rust

import java.io.Closeable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Android-side owner for the Rust headless app runtime.
 *
 * This keeps ViewModels from owning their own effect-drain loops. ViewModels
 * dispatch user intent and render the latest Rust state; platform work remains
 * isolated behind [HeadlessPlatformEnvironment].
 */
class FluxaHeadlessAppRuntime(
    private val engine: FluxaHeadlessEngine,
    private val environment: HeadlessPlatformEnvironment,
    private val maxEffectsPerDispatch: Int = 64
) : Closeable {
    private val mutex = Mutex()
    private val _state = MutableStateFlow<Map<String, Any?>>(emptyMap())
    val state: StateFlow<Map<String, Any?>> = _state.asStateFlow()

    suspend fun dispatch(action: Any): NativeHeadlessEngineResult = mutex.withLock {
        drain(engine.dispatch(action)).also { result ->
            _state.value = result.state
        }
    }

    suspend fun complete(result: HeadlessEffectCompletion): NativeHeadlessEngineResult = mutex.withLock {
        drain(engine.completeEffect(result)).also { next ->
            _state.value = next.state
        }
    }

    private suspend fun drain(initial: NativeHeadlessEngineResult): NativeHeadlessEngineResult {
        var current = initial
        val patches = mutableListOf(current)
        var remaining = maxEffectsPerDispatch
        while (current.effects.isNotEmpty() && remaining > 0) {
            val completion = environment.execute(current.effects.first())
            current = engine.completeEffect(completion)
            patches += current
            remaining--
        }
        return NativeHeadlessEngineResult(
            effects = current.effects,
            stateProvider = { patches.fold(emptyMap<String, Any?>()) { acc, patch -> acc + patch.state } }
        )
    }

    override fun close() {
        (engine as? Closeable)?.close()
    }
}

object FluxaHeadlessRuntimeFactory {
    fun createUniFfi(
        environment: HeadlessPlatformEnvironment,
        initialState: Any = emptyMap<String, Any?>()
    ): FluxaHeadlessAppRuntime {
        return FluxaHeadlessAppRuntime(
            engine = FluxaCoreUniFfi.createHeadlessEngine(initialState),
            environment = environment
        )
    }

    fun createJni(
        environment: HeadlessPlatformEnvironment,
        initialState: Any = emptyMap<String, Any?>()
    ): FluxaHeadlessAppRuntime {
        return FluxaHeadlessAppRuntime(
            engine = FluxaCoreNative.createHeadlessEngine(initialState),
            environment = environment
        )
    }
}
