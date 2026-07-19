package com.fluxa.app.core.rust

import com.fluxa.core.uniffi.PluginHttpClient
import com.fluxa.core.uniffi.appCoreDispatchJson
import com.fluxa.core.uniffi.appCoreStateJson
import com.fluxa.core.uniffi.coreInvoke as coreInvokeUniFfi
import com.fluxa.core.uniffi.createAppCoreStateJson
import com.fluxa.core.uniffi.createHeadlessEngineJson
import com.fluxa.core.uniffi.destroyAppCoreStateJson
import com.fluxa.core.uniffi.destroyHeadlessEngineJson
import com.fluxa.core.uniffi.executePluginScraper as executePluginScraperUniFfi
import com.fluxa.core.uniffi.fluxaCoreVersion
import com.fluxa.core.uniffi.headlessEngineCompleteEffectJson
import com.fluxa.core.uniffi.headlessEngineDispatchJson
import com.fluxa.core.uniffi.headlessEngineSnapshotJson
import com.google.gson.Gson
import com.google.gson.JsonElement
import com.google.gson.JsonParser
import java.io.Closeable

class FluxaUniFfiHeadlessEngineHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable, FluxaHeadlessEngine {
    init {
        check(handle != 0L) { "Fluxa UniFFI headless engine could not be created." }
    }

    fun snapshotJson(): String = headlessEngineSnapshotJson(handle)

    fun dispatchJson(actionJson: String): String = headlessEngineDispatchJson(handle, actionJson)

    override fun dispatch(action: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(dispatchJson(gson.toJson(action)))
    }

    fun completeEffectJson(resultJson: String): String = headlessEngineCompleteEffectJson(handle, resultJson)

    override fun completeEffect(result: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(completeEffectJson(gson.toJson(result)))
    }

    override fun close() {
        destroyHeadlessEngineJson(handle)
    }
}

class FluxaUniFfiCoreStateHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable {
    init {
        check(handle != 0L) { "Fluxa UniFFI core state could not be created." }
    }

    fun snapshotJson(): String = appCoreStateJson(handle)

    fun dispatchJson(actionJson: String): String = appCoreDispatchJson(handle, actionJson)

    fun dispatch(action: Any): String = dispatchJson(gson.toJson(action))

    override fun close() {
        destroyAppCoreStateJson(handle)
    }
}

object FluxaCoreUniFfi {
    private val gson = Gson()

    fun version(): String = fluxaCoreVersion()

    fun createHeadlessEngine(initialState: Any = emptyMap<String, Any?>()): FluxaUniFfiHeadlessEngineHandle {
        return FluxaUniFfiHeadlessEngineHandle(createHeadlessEngineJson(gson.toJson(initialState)), gson)
    }

    fun createAppCoreState(initialState: Any = emptyMap<String, Any?>()): FluxaUniFfiCoreStateHandle {
        return FluxaUniFfiCoreStateHandle(createAppCoreStateJson(gson.toJson(initialState)), gson)
    }

    fun coreInvoke(method: String, argsJson: String): String = coreInvokeUniFfi(method, argsJson)

    fun coreInvokeValue(method: String, argsJson: String): JsonElement {
        val envelope = JsonParser.parseString(coreInvoke(method, argsJson)).asJsonObject
        if (envelope.get("ok")?.asBoolean != true) {
            val error = envelope.getAsJsonObject("error")
            throw IllegalStateException("Fluxa core call '$method' failed: ${error?.get("message")?.asString}")
        }
        return envelope.get("value")
    }

    fun executePluginScraper(
        client: PluginHttpClient,
        code: String,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?
    ): String = executePluginScraperUniFfi(client, code, tmdbId, mediaType, season, episode)
}
