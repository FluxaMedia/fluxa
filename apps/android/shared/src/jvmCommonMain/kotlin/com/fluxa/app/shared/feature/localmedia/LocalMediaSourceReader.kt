package com.fluxa.app.shared.feature.localmedia

import com.fluxa.app.core.rust.FluxaCoreNative
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

fun localMediaContentType(name: String): String = FluxaCoreNative.localMediaContentType(name)
