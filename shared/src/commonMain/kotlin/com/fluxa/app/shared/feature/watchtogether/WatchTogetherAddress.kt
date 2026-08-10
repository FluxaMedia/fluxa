package com.fluxa.app.shared.feature.watchtogether

/** URL/config normalization shared by every platform UI and the coordinator. */
internal object WatchTogetherAddress {
    fun sanitizeConfig(serverUrl: String, serverSecret: String, displayName: String): WatchTogetherConfig =
        WatchTogetherConfig(
            serverUrl = serverUrl.trim(),
            serverSecret = serverSecret.trim(),
            displayName = displayName.trim().ifBlank { "Guest" },
        )

    fun normalizeRoomCode(raw: String): String = raw
        .trim()
        .uppercase()
        .filter(Char::isLetterOrDigit)

    fun isValidRoomCode(code: String): Boolean = code.length in 4..12

    fun websocketUrl(raw: String, secret: String): String? {
        val trimmed = raw.trim().trimEnd('/')
        if (trimmed.isBlank()) return null
        val normalized = when {
            trimmed.startsWith("ws://", true) || trimmed.startsWith("wss://", true) -> trimmed
            trimmed.startsWith("https://", true) -> "wss://${trimmed.substringAfter("://")}".trimEnd('/')
            trimmed.startsWith("http://", true) -> "ws://${trimmed.substringAfter("://")}".trimEnd('/')
            else -> return null
        }
        val base = if (normalized.substringBefore('?').endsWith("/ws")) normalized else {
            val query = normalized.substringAfter('?', missingDelimiterValue = "")
            val path = normalized.substringBefore('?') + "/ws"
            if (query.isBlank()) path else "$path?$query"
        }
        if (secret.isBlank()) return base
        val separator = if ('?' in base) '&' else '?'
        return "$base${separator}token=${encodeQuery(secret)}"
    }

    private fun encodeQuery(value: String): String = buildString {
        value.encodeToByteArray().forEach { byte ->
            val unsigned = byte.toInt() and 0xff
            val ch = unsigned.toChar()
            if (ch.isLetterOrDigit() || ch in "-_.~") append(ch)
            else append('%').append(unsigned.toString(16).uppercase().padStart(2, '0'))
        }
    }
}
