package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformFileStore
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class PlatformPersistentCacheTest {
    @Test
    fun storesValuesUsingPortableSafePaths() = runTest {
        val files = MemoryFileStore()
        val cache = PlatformPersistentCache(files)

        cache.write("manifest", "https://example.com/a?b=1", "payload")

        assertEquals("payload", cache.read("manifest", "https://example.com/a?b=1"))
        assertEquals(setOf("addon_cache/manifest_https___example_com_a_b_1.json"), files.paths)
    }

    @Test
    fun removesStoredValues() = runTest {
        val files = MemoryFileStore()
        val cache = PlatformPersistentCache(files)
        cache.write("user_addons", "profile", "payload")

        cache.remove("user_addons", "profile")

        assertNull(cache.read("user_addons", "profile"))
    }
}

private class MemoryFileStore : PlatformFileStore {
    private val values = mutableMapOf<String, ByteArray>()
    val paths: Set<String> get() = values.keys

    override suspend fun read(path: String): ByteArray? = values[path]
    override suspend fun write(path: String, bytes: ByteArray) {
        values[path] = bytes
    }
    override suspend fun remove(path: String) {
        values.remove(path)
    }
    override suspend fun exists(path: String): Boolean = path in values
}
