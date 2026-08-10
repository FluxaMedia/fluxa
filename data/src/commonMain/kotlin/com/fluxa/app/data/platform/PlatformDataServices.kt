package com.fluxa.app.data.platform

data class PlatformHttpRequest(
    val method: String = "GET",
    val url: String,
    val headers: Map<String, String> = emptyMap(),
    val body: String? = null,
    val timeoutMillis: Long? = null
)

data class PlatformHttpResponse(
    val statusCode: Int,
    val headers: Map<String, String> = emptyMap(),
    val body: String = ""
) {
    val isSuccessful: Boolean get() = statusCode in 200..299
}

interface PlatformHttpClient {
    suspend fun execute(request: PlatformHttpRequest): PlatformHttpResponse
}

interface PlatformKeyValueStore {
    suspend fun read(key: String): String?
    suspend fun write(key: String, value: String)
    suspend fun remove(key: String)
    suspend fun keys(prefix: String): Set<String>
}

interface PlatformSecureStore {
    suspend fun readSecret(key: String): String?
    suspend fun writeSecret(key: String, value: String)
    suspend fun removeSecret(key: String)
}

interface PlatformFileStore {
    suspend fun read(path: String): ByteArray?
    suspend fun write(path: String, bytes: ByteArray)
    suspend fun remove(path: String)
    suspend fun exists(path: String): Boolean
}

interface PlatformClock {
    fun currentTimeMillis(): Long
}

data class PlatformCapabilities(
    val supportsBiometrics: Boolean = false,
    val supportsBackgroundDownloads: Boolean = false,
    val supportsNotifications: Boolean = false,
    val supportsPictureInPicture: Boolean = false,
    val supportsDynamicPlugins: Boolean = false,
    val supportsTorrentPlayback: Boolean = false
)

interface PlatformDataServices {
    val httpClient: PlatformHttpClient
    val keyValueStore: PlatformKeyValueStore
    val secureStore: PlatformSecureStore
    val fileStore: PlatformFileStore
    val clock: PlatformClock
    val capabilities: PlatformCapabilities
}
