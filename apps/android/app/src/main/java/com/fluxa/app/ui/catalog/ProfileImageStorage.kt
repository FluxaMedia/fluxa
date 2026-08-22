package com.fluxa.app.ui.catalog

import android.content.Context
import android.net.Uri
import java.io.File

internal fun copyProfileImageToLocalUri(context: Context, source: Uri): String? {
    return runCatching {
        val mimeType = context.contentResolver.getType(source) ?: "image/jpeg"
        val extension = when {
            mimeType.contains("gif") -> "gif"
            mimeType.contains("png") -> "png"
            mimeType.contains("webp") -> "webp"
            else -> "jpg"
        }
        val directory = File(context.filesDir, "profile_images").apply { mkdirs() }
        val target = File(directory, "profile_${System.currentTimeMillis()}.$extension")
        context.contentResolver.openInputStream(source)?.use { input ->
            target.outputStream().use { output -> input.copyTo(output) }
        } ?: return null
        target.toURI().toString()
    }.getOrNull()
}
