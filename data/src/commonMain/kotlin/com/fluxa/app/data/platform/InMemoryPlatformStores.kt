package com.fluxa.app.data.platform

class InMemoryPlatformKeyValueStore(
    initialValues: Map<String, String> = emptyMap()
) : PlatformKeyValueStore {
    private val values = initialValues.toMutableMap()

    override suspend fun read(key: String): String? = values[key]

    override suspend fun write(key: String, value: String) {
        values[key] = value
    }

    override suspend fun remove(key: String) {
        values.remove(key)
    }

    override suspend fun keys(prefix: String): Set<String> = values.keys.filterTo(mutableSetOf()) { it.startsWith(prefix) }
}

class InMemoryPlatformSecureStore : PlatformSecureStore {
    private val values = mutableMapOf<String, String>()

    override suspend fun readSecret(key: String): String? = values[key]

    override suspend fun writeSecret(key: String, value: String) {
        values[key] = value
    }

    override suspend fun removeSecret(key: String) {
        values.remove(key)
    }
}
