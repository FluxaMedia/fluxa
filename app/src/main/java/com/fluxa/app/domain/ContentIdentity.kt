package com.fluxa.app.domain

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.core.rust.FluxaCoreNative

object ContentIdentity {
    fun traktKey(meta: Meta): String {
        return FluxaCoreNative.contentTraktKey(meta)
    }

    fun billboardKey(meta: Meta): String {
        return FluxaCoreNative.contentBillboardKey(meta)
    }

    fun mergeKeys(meta: Meta): Set<String> {
        return FluxaCoreNative.contentMergeKeys(meta)
    }

    fun watchedKeys(meta: Meta): Set<String> {
        return FluxaCoreNative.contentWatchedKeys(meta)
    }

    fun normalizedBillboardTitle(value: String): String {
        val sb = StringBuilder()
        var lastSpace = false
        for (ch in value.lowercase()) {
            val n = when (ch) {
                'ç' -> 'c'; 'ğ' -> 'g'; 'ı' -> 'i'; 'ö' -> 'o'; 'ş' -> 's'; 'ü' -> 'u'
                else -> if (ch in 'a'..'z' || ch in '0'..'9') ch else ' '
            }
            if (n == ' ') { if (!lastSpace) { sb.append(' '); lastSpace = true } }
            else { sb.append(n); lastSpace = false }
        }
        return sb.toString().trim()
    }
}
