package com.lagradost.cloudstream3

import android.content.Context
import android.content.SharedPreferences
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.json.JsonMapper
import com.fasterxml.jackson.module.kotlin.kotlinModule

/**
 * Fluxa compat stub for CloudStreamApp.
 * Provides the same getKey/setKey/removeKey API that plugins expect,
 * backed by a SharedPreferences store with the same format as CloudStream's DataStore.
 */
class CloudStreamApp {
    companion object {
        private const val PREFERENCES_NAME = "rebuild_preference"

        @PublishedApi
        internal val mapper by lazy {
            JsonMapper.builder()
                .addModule(kotlinModule())
                .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
                .build()
        }

        var context: Context? = null

        @PublishedApi
        internal fun prefs(): SharedPreferences? =
            context?.getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)

        @PublishedApi
        internal fun folderPath(folder: String, path: String) = "${folder.trimEnd('/')}/$path"

        // ── typed get/set used by PreferenceDelegate ──────────────────────────

        fun <T : Any> getKeyClass(path: String, valueType: Class<T>): T? = try {
            val json = prefs()?.getString(path, null) ?: return null
            mapper.readValue(json, valueType)
        } catch (_: Exception) { null }

        fun <T : Any> setKeyClass(path: String, value: T) {
            try {
                prefs()?.edit()?.putString(path, mapper.writeValueAsString(value))?.apply()
            } catch (_: Exception) { }
        }

        // ── setKey ────────────────────────────────────────────────────────────

        fun <T> setKey(path: String, value: T) {
            try {
                prefs()?.edit()?.putString(path, mapper.writeValueAsString(value))?.apply()
            } catch (_: Exception) { }
        }

        fun <T> setKey(folder: String, path: String, value: T) {
            setKey(folderPath(folder, path), value)
        }

        // ── getKey ────────────────────────────────────────────────────────────

        inline fun <reified T : Any> getKey(path: String): T? = try {
            val json = prefs()?.getString(path, null) ?: return null
            mapper.readValue(json, T::class.java)
        } catch (_: Exception) { null }

        inline fun <reified T : Any> getKey(path: String, defVal: T?): T? =
            getKey<T>(path) ?: defVal

        inline fun <reified T : Any> getKey(folder: String, path: String): T? =
            getKey(folderPath(folder, path))

        inline fun <reified T : Any> getKey(folder: String, path: String, defVal: T?): T? =
            getKey<T>(folderPath(folder, path)) ?: defVal

        // ── removeKey / removeKeys / getKeys ──────────────────────────────────

        fun removeKey(path: String) {
            prefs()?.edit()?.remove(path)?.apply()
        }

        fun removeKey(folder: String, path: String) =
            removeKey(folderPath(folder, path))

        fun removeKeys(folder: String): Int? {
            val p = prefs() ?: return null
            val prefix = folder.trimEnd('/') + "/"
            val keys = p.all.keys.filter { it.startsWith(prefix) }
            p.edit().also { ed -> keys.forEach { ed.remove(it) } }.apply()
            return keys.size
        }

        fun getKeys(folder: String): List<String>? {
            val p = prefs() ?: return null
            val prefix = folder.trimEnd('/') + "/"
            return p.all.keys.filter { it.startsWith(prefix) }
        }
    }
}
