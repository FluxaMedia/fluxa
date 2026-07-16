package com.fluxa.app.data.platform

import android.content.SharedPreferences
import java.io.File
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

class AndroidPlatformHttpClient(
    private val client: OkHttpClient
) : PlatformHttpClient {
    override suspend fun execute(request: PlatformHttpRequest): PlatformHttpResponse = withContext(Dispatchers.IO) {
        val effectiveClient = request.timeoutMillis?.let { timeout ->
            client.newBuilder().callTimeout(timeout, TimeUnit.MILLISECONDS).build()
        } ?: client
        val body = request.body?.toRequestBody(
            request.headers.entries.firstOrNull { it.key.equals("Content-Type", true) }?.value?.toMediaTypeOrNull()
        )
        val nativeRequest = Request.Builder()
            .url(request.url)
            .apply { request.headers.forEach { (name, value) -> header(name, value) } }
            .method(request.method, body)
            .build()
        effectiveClient.newCall(nativeRequest).execute().use { response ->
            PlatformHttpResponse(
                statusCode = response.code,
                headers = response.headers.toMultimap().mapValues { it.value.joinToString(",") },
                body = response.body.string()
            )
        }
    }
}

class AndroidPlatformKeyValueStore(
    private val preferences: SharedPreferences
) : PlatformKeyValueStore {
    override suspend fun read(key: String): String? = preferences.getString(key, null)

    override suspend fun write(key: String, value: String) {
        preferences.edit().putString(key, value).apply()
    }

    override suspend fun remove(key: String) {
        preferences.edit().remove(key).apply()
    }

    override suspend fun keys(prefix: String): Set<String> = preferences.all.keys.filterTo(mutableSetOf()) { it.startsWith(prefix) }
}

class AndroidPlatformFileStore(
    private val root: File
) : PlatformFileStore {
    private fun resolve(path: String): File {
        val file = File(root, path).canonicalFile
        require(file.path == root.canonicalPath || file.path.startsWith(root.canonicalPath + File.separator))
        return file
    }

    override suspend fun read(path: String): ByteArray? = withContext(Dispatchers.IO) {
        resolve(path).takeIf(File::isFile)?.readBytes()
    }

    override suspend fun write(path: String, bytes: ByteArray) = withContext(Dispatchers.IO) {
        resolve(path).apply { parentFile?.mkdirs() }.writeBytes(bytes)
    }

    override suspend fun remove(path: String) {
        withContext(Dispatchers.IO) { resolve(path).delete() }
    }

    override suspend fun exists(path: String): Boolean = withContext(Dispatchers.IO) { resolve(path).exists() }
}

object AndroidPlatformClock : PlatformClock {
    override fun currentTimeMillis(): Long = System.currentTimeMillis()
}
