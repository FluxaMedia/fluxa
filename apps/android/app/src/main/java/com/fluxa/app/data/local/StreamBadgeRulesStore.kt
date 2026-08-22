package com.fluxa.app.data.local

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreNative

object StreamBadgeRulesStore {
    private const val PREFS_NAME = "fluxa_stream_badges"
    private const val KEY_RULES = "rules_json"
    private const val KEY_PLACEMENT = "placement"
    private val EMPTY_RULES = """{"imports":[]}"""

    fun read(context: Context): String {
        val stored = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .getString(KEY_RULES, null) ?: EMPTY_RULES
        return runCatching { FluxaCoreNative.normalizeStreamBadgeRules(stored) }.getOrDefault(stored)
    }

    fun write(context: Context, rulesJson: String) {
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_RULES, rulesJson)
            .apply()
    }

    fun importFromUrl(context: Context, sourceUrl: String, payload: String): String {
        val importJson = FluxaCoreNative.parseStreamBadgeImport(sourceUrl, payload)
        val nextRules = FluxaCoreNative.upsertStreamBadgeImport(read(context), importJson, true)
        write(context, nextRules)
        return nextRules
    }

    fun setActiveSource(context: Context, sourceUrl: String) {
        write(context, FluxaCoreNative.setActiveStreamBadgeSource(read(context), sourceUrl))
    }

    fun removeSource(context: Context, sourceUrl: String) {
        write(context, FluxaCoreNative.removeStreamBadgeSource(read(context), sourceUrl))
    }

    fun readPlacement(context: Context): String {
        val stored = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).getString(KEY_PLACEMENT, "bottom")
        return if (stored == "top") "top" else "bottom"
    }

    fun writePlacement(context: Context, placement: String) {
        val normalized = if (placement == "top") "top" else "bottom"
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_PLACEMENT, normalized)
            .apply()
    }
}
