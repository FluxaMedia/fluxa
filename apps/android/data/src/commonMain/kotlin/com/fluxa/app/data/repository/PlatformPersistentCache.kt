package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformFileStore

class PlatformPersistentCache(
    private val fileStore: PlatformFileStore,
    private val directory: String = "addon_cache"
) {
    suspend fun read(namespace: String, key: String): String? {
        return fileStore.read(path(namespace, key))?.decodeToString()
    }

    suspend fun write(namespace: String, key: String, value: String) {
        fileStore.write(path(namespace, key), value.encodeToByteArray())
    }

    suspend fun remove(namespace: String, key: String) {
        fileStore.remove(path(namespace, key))
    }

    private fun path(namespace: String, key: String): String {
        return "$directory/${safePart(namespace)}_${safePart(key)}.json"
    }

    private fun safePart(value: String): String = value
        .map { character -> if (character.isLetterOrDigit() || character == '_' || character == '-') character else '_' }
        .joinToString("")
        .take(200)
}
