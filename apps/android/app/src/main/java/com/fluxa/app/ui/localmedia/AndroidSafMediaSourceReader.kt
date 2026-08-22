package com.fluxa.app.ui.localmedia

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.shared.feature.localmedia.LocalMediaFileCandidate
import com.fluxa.app.shared.feature.localmedia.LocalMediaOpenedStream
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceConfig
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceReader
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceType
import com.fluxa.app.shared.feature.localmedia.localMediaContentType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class AndroidSafMediaSourceReader(
    context: Context,
) : LocalMediaSourceReader {
    private val appContext = context.applicationContext

    override fun supports(type: LocalMediaSourceType): Boolean = type == LocalMediaSourceType.LocalFolder

    override suspend fun listFiles(source: LocalMediaSourceConfig): List<LocalMediaFileCandidate> = withContext(Dispatchers.IO) {
        val root = DocumentFile.fromTreeUri(appContext, Uri.parse(source.location))
            ?: error("Cannot access selected media folder")
        val out = ArrayList<LocalMediaFileCandidate>()
        walk(root, emptyList(), out, 0)
        out
    }

    private fun walk(
        directory: DocumentFile,
        parents: List<String>,
        out: MutableList<LocalMediaFileCandidate>,
        depth: Int,
    ) {
        if (depth > 24 || out.size >= 50_000) return
        for (child in directory.listFiles()) {
            val name = child.name.orEmpty()
            if (name.startsWith('.')) continue
            when {
                child.isDirectory -> walk(child, (listOf(name) + parents).take(4), out, depth + 1)
                child.isFile && FluxaCoreNative.localMediaIsVideoFile(name) -> out += LocalMediaFileCandidate(
                    locator = child.uri.toString(),
                    displayName = name,
                    parentHints = parents,
                    sizeBytes = child.length().coerceAtLeast(0L),
                    modifiedAtMs = child.lastModified().coerceAtLeast(0L),
                )
            }
        }
    }

    override fun open(source: LocalMediaSourceConfig, locator: String, offset: Long): LocalMediaOpenedStream {
        val uri = Uri.parse(locator)
        val descriptor = appContext.contentResolver.openAssetFileDescriptor(uri, "r")
            ?: error("Cannot open local media URI")
        val input = descriptor.createInputStream()
        var remaining = offset.coerceAtLeast(0L)
        while (remaining > 0L) {
            val skipped = input.skip(remaining)
            if (skipped <= 0L) break
            remaining -= skipped
        }
        val length = descriptor.length.takeIf { it >= 0L } ?: 0L
        val wrapped = object : java.io.FilterInputStream(input) {
            override fun close() {
                try { super.close() } finally { descriptor.close() }
            }
        }
        return LocalMediaOpenedStream(wrapped, length, appContext.contentResolver.getType(uri) ?: localMediaContentType(locator))
    }
}
