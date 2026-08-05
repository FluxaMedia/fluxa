package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformFileStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

class DesktopPlatformFileStore(
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

    override suspend fun remove(path: String) = withContext(Dispatchers.IO) {
        resolve(path).delete()
        Unit
    }

    override suspend fun exists(path: String): Boolean = withContext(Dispatchers.IO) {
        resolve(path).exists()
    }
}
