package com.fluxa.app.core.rust

import com.google.gson.Gson
import java.io.Closeable

class FluxaHeadlessEngineHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable, FluxaHeadlessEngine {
    init {
        check(handle != 0L) { "Fluxa headless engine could not be created." }
    }

    fun snapshotJson(): String = FluxaCoreNative.headlessEngineSnapshotJson(handle)

    fun dispatchJson(actionJson: String): String = FluxaCoreNative.headlessEngineDispatchJson(handle, actionJson)

    override fun dispatch(action: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(dispatchJson(gson.toJson(action)))
    }

    fun completeEffectJson(resultJson: String): String = FluxaCoreNative.headlessEngineCompleteEffectJson(handle, resultJson)

    override fun completeEffect(result: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(completeEffectJson(gson.toJson(result)))
    }

    override fun close() {
        FluxaCoreNative.destroyHeadlessEngine(handle)
    }
}

class FluxaCoreStateHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable {
    init {
        check(handle != 0L) { "Fluxa core state could not be created." }
    }

    fun snapshotJson(): String = FluxaCoreNative.appCoreStateJson(handle)

    fun dispatchJson(actionJson: String): String = FluxaCoreNative.appCoreDispatchJson(handle, actionJson)

    fun dispatch(action: Any): String = dispatchJson(gson.toJson(action))

    override fun close() {
        FluxaCoreNative.destroyAppCoreState(handle)
    }
}

