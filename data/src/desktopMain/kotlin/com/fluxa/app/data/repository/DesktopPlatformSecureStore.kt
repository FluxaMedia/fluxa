package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformSecureStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.io.File
import java.util.Properties

class DesktopPlatformSecureStore(
    private val file: File
) : PlatformSecureStore {
    private val mutex = Mutex()

    private fun loadProperties(): Properties = Properties().apply {
        if (file.isFile) file.inputStream().use { load(it) }
    }

    private fun saveProperties(properties: Properties) {
        file.parentFile?.mkdirs()
        file.outputStream().use { properties.store(it, null) }
        file.setReadable(false, false)
        file.setWritable(false, false)
        file.setReadable(true, true)
        file.setWritable(true, true)
    }

    override suspend fun readSecret(key: String): String? = withContext(Dispatchers.IO) {
        mutex.withLock { loadProperties().getProperty(key) }
    }

    override suspend fun writeSecret(key: String, value: String) = withContext(Dispatchers.IO) {
        mutex.withLock {
            val properties = loadProperties()
            properties.setProperty(key, value)
            saveProperties(properties)
        }
    }

    override suspend fun removeSecret(key: String) = withContext(Dispatchers.IO) {
        mutex.withLock {
            val properties = loadProperties()
            properties.remove(key)
            saveProperties(properties)
        }
    }
}
