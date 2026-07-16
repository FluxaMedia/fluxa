package com.fluxa.app.domain.discovery

object StremioResourceUrlBuilder {
    fun normalizeManifestUrl(rawUrl: String): String {
        val trimmed = rawUrl.trim()
        if (trimmed.isEmpty()) return ""
        val withScheme = when {
            trimmed.startsWith("stremio://", true) -> "https://${trimmed.drop(10)}"
            trimmed.startsWith("http://", true) || trimmed.startsWith("https://", true) -> trimmed
            else -> "https://$trimmed"
        }
        if (withScheme.endsWith("manifest.json", true)) return withScheme
        return if (withScheme.endsWith('/')) "${withScheme}manifest.json" else "$withScheme/manifest.json"
    }

    fun resourceUrl(transportUrl: String, resource: String, contentType: String, id: String): String {
        return "${baseUrl(transportUrl)}${encode(resource)}/${encode(contentType)}/${encode(id)}.json"
    }

    fun catalogUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        extraName: String? = null,
        extraValue: String? = null
    ): String {
        val extra = if (!extraName.isNullOrBlank() && extraValue != null) {
            "/${encode(extraName)}=${encode(extraValue)}"
        } else {
            ""
        }
        return "${baseUrl(transportUrl)}catalog/${encode(contentType)}/${encode(catalogId)}$extra.json"
    }

    private fun baseUrl(transportUrl: String): String {
        val manifestIndex = transportUrl.lastIndexOf("manifest.json", ignoreCase = true)
        val base = if (manifestIndex >= 0) transportUrl.substring(0, manifestIndex) else transportUrl
        return if (base.endsWith('/')) base else "$base/"
    }

    private fun encode(value: String): String = buildString {
        value.encodeToByteArray().forEach { byte ->
            val unsigned = byte.toInt() and 0xff
            val allowed = unsigned in 'a'.code..'z'.code || unsigned in 'A'.code..'Z'.code ||
                unsigned in '0'.code..'9'.code || unsigned == '-'.code || unsigned == '_'.code ||
                unsigned == '.'.code || unsigned == '*'.code
            if (allowed) {
                append(unsigned.toChar())
            } else {
                append('%')
                append(Hex[unsigned ushr 4])
                append(Hex[unsigned and 0x0f])
            }
        }
    }

    private const val Hex = "0123456789ABCDEF"
}
