package com.fluxa.app.core.rust

data class NativeHeadlessEffect(
    val id: String = "",
    val type: String = "",
    val generation: Long = 0L,
    val payload: Map<String, Any?> = emptyMap()
)

data class HeadlessEffectCompletion(
    val effectId: String,
    val status: String,
    val value: Any? = null,
    val error: Any? = null
)

interface HeadlessPlatformEnvironment {
    suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion
}

fun interface HeadlessEffectHandler {
    suspend fun execute(effect: NativeHeadlessEffect): Any?
}

class HeadlessEffectEnvironment(
    private val handlers: Map<String, HeadlessEffectHandler>
) : HeadlessPlatformEnvironment {
    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val handler = handlers[effect.type]
            ?: return HeadlessEffectCompletion(effect.id, "error", error = mapOf("code" to "unsupported_effect"))
        return try {
            HeadlessEffectCompletion(effect.id, "ok", value = handler.execute(effect))
        } catch (cause: Throwable) {
            HeadlessEffectCompletion(
                effect.id,
                "error",
                error = mapOf("code" to "effect_failed", "message" to cause.message.orEmpty())
            )
        }
    }
}

data class NativeCoreCapabilitySet(
    val http: Boolean = false,
    val storage: Boolean = false,
    val auth: Boolean = false,
    val player: Boolean = false,
    val plugins: Boolean = false,
    val torrent: Boolean = false,
    val localStream: Boolean = false,
    val notifications: Boolean = false
)
