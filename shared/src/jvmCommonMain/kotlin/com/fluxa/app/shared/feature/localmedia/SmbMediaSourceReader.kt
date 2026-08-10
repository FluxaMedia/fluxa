package com.fluxa.app.shared.feature.localmedia

import jcifs.CIFSContext
import jcifs.context.SingletonContext
import jcifs.smb.NtlmPasswordAuthenticator
import jcifs.smb.SmbFile
import jcifs.smb.SmbFileInputStream
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SmbMediaSourceReader : LocalMediaSourceReader {
    override fun supports(type: LocalMediaSourceType): Boolean = type == LocalMediaSourceType.Smb

    override suspend fun listFiles(source: LocalMediaSourceConfig): List<LocalMediaFileCandidate> = withContext(Dispatchers.IO) {
        val context = contextFor(source)
        val rootUrl = normalizeRoot(source.location)
        val root = SmbFile(rootUrl, context)
        val out = ArrayList<LocalMediaFileCandidate>()
        walk(root, root, out, depth = 0)
        out
    }

    private fun walk(
        root: SmbFile,
        current: SmbFile,
        out: MutableList<LocalMediaFileCandidate>,
        depth: Int,
    ) {
        if (depth > 24 || out.size >= 50_000) return
        val children = current.listFiles()
        for (child in children) {
            val name = child.name.trimEnd('/')
            if (name.startsWith('.')) continue
            if (runCatching { child.isDirectory }.getOrDefault(false)) {
                walk(root, child, out, depth + 1)
            } else if (LocalMediaFilenameParser.isVideoFile(name)) {
                val relative = child.path.removePrefix(root.path).trim('/')
                val parents = relative.split('/').dropLast(1).asReversed().take(4)
                out += LocalMediaFileCandidate(
                    locator = child.path,
                    displayName = name,
                    parentHints = parents,
                    sizeBytes = runCatching { child.length() }.getOrDefault(0L),
                    modifiedAtMs = runCatching { child.lastModified() }.getOrDefault(0L),
                )
            }
        }
    }

    override fun open(source: LocalMediaSourceConfig, locator: String, offset: Long): LocalMediaOpenedStream {
        val context = contextFor(source)
        val file = SmbFile(locator, context)
        val length = runCatching { file.length() }.getOrDefault(0L)
        val stream = SmbFileInputStream(file)
        var remaining = offset.coerceAtLeast(0L)
        while (remaining > 0L) {
            val skipped = stream.skip(remaining)
            if (skipped <= 0L) break
            remaining -= skipped
        }
        return LocalMediaOpenedStream(stream, length, localMediaContentType(file.name))
    }

    private fun contextFor(source: LocalMediaSourceConfig): CIFSContext {
        val base = SingletonContext.getInstance()
        val username = source.username.orEmpty()
        if (username.isBlank()) return base
        val (domain, user) = username.split('\\', limit = 2).let { parts ->
            if (parts.size == 2) parts[0] to parts[1] else "" to parts[0]
        }
        return base.withCredentials(NtlmPasswordAuthenticator(domain, user, source.password.orEmpty()))
    }

    private fun normalizeRoot(value: String): String {
        val root = if (value.startsWith("smb://", ignoreCase = true)) value else "smb://$value"
        return if (root.endsWith('/')) root else "$root/"
    }
}
