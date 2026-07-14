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
