package com.fluxa.app.domain

import com.fluxa.app.data.remote.Meta

private val imdbIdRegex = Regex("tt\\d+")

fun contentBillboardKey(meta: Meta): String {
    val imdb = imdbIdRegex.find(meta.id)?.value
    if (imdb != null) return "${meta.type}:$imdb"
    val name = meta.originalName?.takeIf { it.isNotEmpty() } ?: meta.name
    val year = meta.releaseInfo?.takeIf { it.length >= 4 }?.substring(0, 4)
        ?: meta.released?.takeIf { it.length >= 4 }?.substring(0, 4)
        ?: ""
    return "${meta.type}:${normalizedBillboardTitle(name)}:$year"
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
