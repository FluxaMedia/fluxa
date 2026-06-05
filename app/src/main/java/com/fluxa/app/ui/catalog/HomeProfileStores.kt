package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SearchHistoryStore @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs = context.getSharedPreferences("fluxa_search", Context.MODE_PRIVATE)
    // ... rest of implementation remains same
    fun load(profile: UserProfile?): List<Meta> {
        val saved = prefs.getString(key(profile), null) ?: return emptyList()
        return runCatching {
            val listType = object : TypeToken<List<Meta>>() {}.type
            gson.fromJson<List<Meta>>(saved, listType)
                ?.filter { it.id.isNotBlank() && it.name.isNotBlank() }
                .orEmpty()
        }.getOrElse { emptyList() }
    }

    fun save(list: List<Meta>, profile: UserProfile?) {
        prefs.edit().putString(key(profile), gson.toJson(list)).apply()
    }

    private fun key(profile: UserProfile?): String {
        return if (profile?.isGuest == true || profile == null) {
            "history_guest"
        } else {
            "history_${profile.id}"
        }
    }
}

@Singleton
class ForgottenContinueWatchingStore @Inject constructor(@ApplicationContext context: Context) {
    private val prefs = context.getSharedPreferences("fluxa_search", Context.MODE_PRIVATE)

    fun load(profile: UserProfile?): Set<String> {
        return prefs.getStringSet(key(profile), emptySet()).orEmpty()
    }

    fun save(profile: UserProfile?, keys: Set<String>) {
        prefs.edit().putStringSet(key(profile), keys).apply()
    }

    private fun key(profile: UserProfile?): String {
        return if (profile?.isGuest == true || profile == null) {
            "forgotten_continue_guest"
        } else {
            "forgotten_continue_${profile.id}"
        }
    }
}

@Singleton
class HomeCategoryCache @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs = context.getSharedPreferences("fluxa_home_cache", Context.MODE_PRIVATE)

    fun load(profile: UserProfile?): List<HomeCategory> {
        val saved = prefs.getString(key(profile), null) ?: return emptyList()
        return runCatching {
            val listType = object : TypeToken<List<HomeCategory>>() {}.type
            gson.fromJson<List<HomeCategory>>(saved, listType).orEmpty()
        }.getOrElse { emptyList() }
    }

    fun save(profile: UserProfile?, list: List<HomeCategory>) {
        prefs.edit().putString(key(profile), gson.toJson(list)).apply()
    }

    private fun key(profile: UserProfile?): String {
        return if (profile?.isGuest == true || profile == null) "home_cache_guest" else "home_cache_${profile.id}"
    }
}
