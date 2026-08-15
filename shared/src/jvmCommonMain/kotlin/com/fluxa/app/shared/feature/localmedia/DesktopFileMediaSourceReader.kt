package com.fluxa.app.shared.feature.localmedia

import com.fluxa.app.core.rust.FluxaCoreNative
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream

/** JVM filesystem reader used by Desktop and by OS-mounted NAS shares. */
class DesktopFileMediaSourceReader : LocalMediaSourceReader {
    override fun supports(type: LocalMediaSourceType): Boolean = type == LocalMediaSourceType.LocalFolder

    override suspend fun listFiles(source: LocalMediaSourceConfig): List<LocalMediaFileCandidate> = withContext(Dispatchers.IO) {
        val root = File(source.location.removePrefix("file://"))
        require(root.isDirectory) { "Media folder is unavailable: ${source.displayName}" }
        root.walkTopDown()
            .onEnter { dir -> !dir.name.startsWith('.') }
            .filter { it.isFile && FluxaCoreNative.localMediaIsVideoFile(it.name) }
            .map { file ->
                val parentHints = generateSequence(file.parentFile) { it.parentFile }
                    .takeWhile { it.absolutePath.startsWith(root.absolutePath) }
                    .map { it.name }
                    .filter(String::isNotBlank)
                    .take(4)
                    .toList()
                LocalMediaFileCandidate(
                    locator = file.absolutePath,
                    displayName = file.name,
                    parentHints = parentHints,
                    sizeBytes = file.length(),
                    modifiedAtMs = file.lastModified(),
                )
            }
            .toList()
    }

    override fun open(source: LocalMediaSourceConfig, locator: String, offset: Long): LocalMediaOpenedStream {
        val file = File(locator)
        val stream = FileInputStream(file)
        if (offset > 0L) stream.channel.position(offset.coerceAtMost(file.length()))
        return LocalMediaOpenedStream(stream, file.length(), localMediaContentType(file.name))
    }
}
