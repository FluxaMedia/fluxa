package com.fluxa.app.shared.feature.localmedia

import java.io.InputStream

data class LocalMediaFileCandidate(
    val locator: String,
    val displayName: String,
    val parentHints: List<String>,
    val sizeBytes: Long,
    val modifiedAtMs: Long,
)

data class LocalMediaOpenedStream(
    val input: InputStream,
    val totalLength: Long,
    val contentType: String,
)

interface LocalMediaSourceReader {
    fun supports(type: LocalMediaSourceType): Boolean
    suspend fun listFiles(source: LocalMediaSourceConfig): List<LocalMediaFileCandidate>
    fun open(source: LocalMediaSourceConfig, locator: String, offset: Long): LocalMediaOpenedStream
}

fun localMediaContentType(name: String): String = when (name.substringAfterLast('.', "").lowercase()) {
    "mp4", "m4v" -> "video/mp4"
    "webm" -> "video/webm"
    "ts", "m2ts" -> "video/mp2t"
    "mov" -> "video/quicktime"
    "avi" -> "video/x-msvideo"
    "mkv" -> "video/x-matroska"
    else -> "application/octet-stream"
}
