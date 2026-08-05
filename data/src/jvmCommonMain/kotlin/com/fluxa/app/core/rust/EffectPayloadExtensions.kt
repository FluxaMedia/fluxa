package com.fluxa.app.core.rust

import com.fluxa.app.data.local.UserProfile
import com.google.gson.Gson

internal fun Map<String, Any?>.string(key: String, default: String = ""): String =
    stringOrNull(key) ?: default

internal fun Map<String, Any?>.stringOrNull(key: String): String? =
    this[key]?.takeUnless { it == Unit }?.toString()?.takeIf { it != "null" && it.isNotBlank() }

internal fun Map<String, Any?>.boolean(key: String, default: Boolean = false): Boolean =
    when (val value = this[key]) {
        is Boolean -> value
        is String -> value.toBooleanStrictOrNull() ?: default
        else -> default
    }

internal fun Map<String, Any?>.number(key: String): Number? = this[key] as? Number

internal fun Map<String, Any?>.list(key: String): List<Any?> = this[key] as? List<Any?> ?: emptyList()

internal fun Map<String, Any?>.objectValue(key: String): Map<String, Any?>? {
    @Suppress("UNCHECKED_CAST")
    return this[key] as? Map<String, Any?>
}

internal fun Map<String, Any?>.extraString(key: String): String? = objectValue("extra")?.stringOrNull(key)

internal fun Map<String, Any?>.extraNumber(key: String): Number? = objectValue("extra")?.number(key)

internal fun <T> Map<String, T>.mapKeysNotNullToInt(): Map<Int, T> =
    mapNotNull { (key, value) -> key.toIntOrNull()?.let { it to value } }.toMap()

internal fun Map<String, Any?>.parseProfile(gson: Gson): UserProfile? {
    val raw = objectValue("profile")?.takeIf { it.isNotEmpty() } ?: return null
    return runCatching { gson.fromJson(gson.toJsonTree(raw), UserProfile::class.java) }.getOrNull()
}
