package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformKeyValueStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.io.File
import java.util.Properties

class DesktopPlatformKeyValueStore(
    private val file: File
) : PlatformKeyValueStore {
    private val mutex = Mutex()

    private fun loadProperties(): Properties = Properties().apply {
        if (file.isFile) file.inputStream().use { load(it) }
    }

    private fun saveProperties(properties: Properties) {
        file.parentFile?.mkdirs()
        file.outputStream().use { properties.store(it, null) }
    }

    override suspend fun read(key: String): String? = withContext(Dispatchers.IO) {
        mutex.withLock { loadProperties().getProperty(key) }
    }

    override suspend fun write(key: String, value: String) = withContext(Dispatchers.IO) {
        mutex.withLock {
            val properties = loadProperties()
            properties.setProperty(key, value)
            saveProperties(properties)
        }
    }

    override suspend fun remove(key: String) = withContext(Dispatchers.IO) {
        mutex.withLock {
            val properties = loadProperties()
            properties.remove(key)
            saveProperties(properties)
        }
    }

    override suspend fun keys(prefix: String): Set<String> = withContext(Dispatchers.IO) {
        mutex.withLock { loadProperties().stringPropertyNames().filterTo(mutableSetOf()) { it.startsWith(prefix) } }
    }
}
