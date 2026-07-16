package com.fluxa.app.player

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

object LastMediaDebugInfoStore {
    private const val PREFS = "last_media_debug_info"
    private const val KEY_URL = "url"
    private const val KEY_TITLE = "title"
    private const val KEY_INFO = "technical_info"
    private const val KEY_UPDATED_AT = "updated_at"
    private const val MIN_WRITE_INTERVAL_MS = 1_500L

    private val _state = MutableStateFlow(LastMediaDebugInfo())
    val state: StateFlow<LastMediaDebugInfo> = _state

    @Volatile private var initialized = false
    @Volatile private var lastWriteAtMs = 0L

    fun initialize(context: Context) {
        if (initialized) return
        synchronized(this) {
            if (initialized) return
            val prefs = context.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            _state.value = LastMediaDebugInfo(
                url = prefs.getString(KEY_URL, "").orEmpty(),
                title = prefs.getString(KEY_TITLE, "").orEmpty(),
                technicalInfo = prefs.getString(KEY_INFO, "").orEmpty(),
                updatedAtMs = prefs.getLong(KEY_UPDATED_AT, 0L)
            )
            initialized = true
        }
    }

    fun save(context: Context, url: String?, title: String?, technicalInfo: String?) {
        initialize(context)
        val info = technicalInfo?.takeIf { it.isNotBlank() } ?: return
        val now = System.currentTimeMillis()
        val previous = _state.value
        if (previous.technicalInfo == info && previous.url == url.orEmpty()) return
        if (now - lastWriteAtMs < MIN_WRITE_INTERVAL_MS && previous.hasInfo) {
            _state.value = previous.copy(
                url = url.orEmpty(),
                title = title.orEmpty(),
                technicalInfo = info,
                updatedAtMs = now
            )
            return
        }
        val next = LastMediaDebugInfo(
            url = url.orEmpty(),
            title = title.orEmpty(),
            technicalInfo = info,
            updatedAtMs = now
        )
        _state.value = next
        lastWriteAtMs = now
        context.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_URL, next.url)
            .putString(KEY_TITLE, next.title)
            .putString(KEY_INFO, next.technicalInfo)
            .putLong(KEY_UPDATED_AT, next.updatedAtMs)
            .apply()
    }

    fun clear(context: Context) {
        initialize(context)
        _state.value = LastMediaDebugInfo()
        context.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .apply()
    }

    fun formattedUpdatedAt(value: Long): String {
        if (value <= 0L) return ""
        return SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).format(Date(value))
    }
}
