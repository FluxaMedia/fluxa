package com.lagradost.cloudstream3.utils

import android.content.Context
import android.content.SharedPreferences
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.json.JsonMapper
import com.fasterxml.jackson.module.kotlin.kotlinModule
import com.lagradost.cloudstream3.CloudStreamApp

/**
 * Fluxa compat stub for CloudStream's DataStore.
 * Plugins compiled against CloudStream call these functions for persistent key-value storage.
 * Uses the same SharedPreferences name and JSON format as the real CloudStream DataStore.
 */

const val PREFERENCES_NAME = "rebuild_preference"

object DataStore {
    val mapper: JsonMapper by lazy {
        JsonMapper.builder()
            .addModule(kotlinModule())
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
            .build()
    }

    fun getFolderName(folder: String, path: String): String = "${folder.trimEnd('/')}/$path"

    fun Context.getSharedPrefs(): SharedPreferences =
        getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)

    @Suppress("DEPRECATION")
    fun Context.getDefaultSharedPrefs(): SharedPreferences =
        android.preference.PreferenceManager.getDefaultSharedPreferences(this)

    fun Context.getKeys(folder: String): List<String> {
        val prefix = folder.trimEnd('/') + "/"
        return getSharedPrefs().all.keys.filter { it.startsWith(prefix) }
    }

    fun Context.containsKey(path: String): Boolean =
        getSharedPrefs().contains(path)

    fun Context.containsKey(folder: String, path: String): Boolean =
        containsKey(getFolderName(folder, path))

    fun Context.removeKey(path: String) {
        try {
            val prefs = getSharedPrefs()
            if (prefs.contains(path)) prefs.edit().remove(path).apply()
        } catch (_: Exception) {}
    }

    fun Context.removeKey(folder: String, path: String) =
        removeKey(getFolderName(folder, path))

    fun Context.removeKeys(folder: String): Int = try {
        val keys = getKeys(folder)
        val editor = getSharedPrefs().edit()
        keys.forEach { editor.remove(it) }
        editor.apply()
        keys.size
    } catch (_: Exception) { 0 }

    fun <T> Context.setKey(path: String, value: T) = try {
        getSharedPrefs().edit().putString(path, mapper.writeValueAsString(value)).apply()
    } catch (_: Exception) {}

    fun <T> Context.setKey(folder: String, path: String, value: T) =
        setKey(getFolderName(folder, path), value)

    fun <T> Context.getKey(path: String, valueType: Class<T>): T? = try {
        val json = getSharedPrefs().getString(path, null) ?: return null
        mapper.readValue(json, valueType)
    } catch (_: Exception) { null }

    inline fun <reified T : Any> Context.getKey(path: String): T? = try {
        val json = getSharedPrefs().getString(path, null) ?: return null
        mapper.readValue(json, T::class.java)
    } catch (_: Exception) { null }

    inline fun <reified T : Any> Context.getKey(path: String, defVal: T?): T? =
        getKey<T>(path) ?: defVal

    inline fun <reified T : Any> Context.getKey(folder: String, path: String): T? =
        getKey(getFolderName(folder, path))

    inline fun <reified T : Any> Context.getKey(folder: String, path: String, defVal: T?): T? =
        getKey<T>(getFolderName(folder, path)) ?: defVal

    inline fun <reified T : Any> String.toKotlinObject(): T =
        mapper.readValue(this, T::class.java)

    fun <T> String.toKotlinObject(valueType: Class<T>): T =
        mapper.readValue(this, valueType)
}

// ── Top-level extension shims ─────────────────────────────────────────────────
// Plugins often import these as top-level functions from CloudStreamApp

fun <T> Context.setKey(path: String, value: T) = DataStore.run { setKey(path, value) }
fun <T> Context.setKey(folder: String, path: String, value: T) = DataStore.run { setKey(folder, path, value) }

inline fun <reified T : Any> Context.getKey(path: String): T? = DataStore.run { getKey(path) }
inline fun <reified T : Any> Context.getKey(path: String, defVal: T?): T? = DataStore.run { getKey(path, defVal) }
inline fun <reified T : Any> Context.getKey(folder: String, path: String): T? = DataStore.run { getKey(folder, path) }
inline fun <reified T : Any> Context.getKey(folder: String, path: String, defVal: T?): T? = DataStore.run { getKey(folder, path, defVal) }

fun Context.removeKey(path: String) = DataStore.run { removeKey(path) }
fun Context.removeKey(folder: String, path: String) = DataStore.run { removeKey(folder, path) }
fun Context.removeKeys(folder: String): Int = DataStore.run { removeKeys(folder) }
fun Context.getKeys(folder: String): List<String> = DataStore.run { getKeys(folder) }
fun Context.containsKey(path: String): Boolean = DataStore.run { containsKey(path) }
fun Context.containsKey(folder: String, path: String): Boolean = DataStore.run { containsKey(folder, path) }
