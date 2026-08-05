package com.fluxa.app.data.local

import android.content.Context
import android.content.SharedPreferences
import androidx.annotation.WorkerThread
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProfilePickerSettingsStore @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs: SharedPreferences = context.getSharedPreferences("fluxa_profile_picker", Context.MODE_PRIVATE)
    private val lock = Any()

    fun get(): ProfilePickerSettingsRecord {
        val json = prefs.getString(KEY, null) ?: return ProfilePickerSettingsRecord()
        return runCatching { gson.fromJson(json, ProfilePickerSettingsRecord::class.java) }.getOrNull()
            ?: ProfilePickerSettingsRecord()
    }

    @WorkerThread
    fun save(record: ProfilePickerSettingsRecord) {
        synchronized(lock) {
            prefs.edit().putString(KEY, gson.toJson(record)).apply()
        }
    }

    fun registerOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.registerOnSharedPreferenceChangeListener(listener)
    }

    fun unregisterOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.unregisterOnSharedPreferenceChangeListener(listener)
    }

    private companion object {
        const val KEY = "picker_settings"
    }
}
