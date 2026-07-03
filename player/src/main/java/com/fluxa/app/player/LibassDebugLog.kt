package com.fluxa.app.player

import android.net.Uri
import android.util.Log
import androidx.media3.common.Format

object LibassDebugLog {
    const val TAG = "FluxaLibassExo"

    fun d(message: String) {
        Log.d(TAG, message)
    }

    fun w(message: String, throwable: Throwable? = null) {
        if (throwable == null) Log.w(TAG, message) else Log.w(TAG, message, throwable)
    }

    fun urlSummary(rawUrl: String?): String {
        if (rawUrl.isNullOrBlank()) return "<empty>"
        return runCatching {
            val uri = Uri.parse(rawUrl)
            val scheme = uri.scheme.orEmpty().ifBlank { "unknown" }
            val host = uri.host.orEmpty()
            val path = uri.path.orEmpty()
            val name = path.substringAfterLast('/').ifBlank { path.takeLast(32) }
            buildString {
                append(scheme)
                if (host.isNotBlank()) append("://").append(host)
                if (name.isNotBlank()) append("/.../").append(name)
            }
        }.getOrElse {
            rawUrl.substringBefore('?').substringBefore('#').takeLast(80)
        }
    }

    fun formatSummary(format: Format?): String {
        if (format == null) return "<none>"
        return "id=${format.id} mime=${format.sampleMimeType} label=${format.label} lang=${format.language}"
    }
}
