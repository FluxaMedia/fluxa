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
    fun load(profile: UserProfile?): List<Meta> {
        val key = key(profile) ?: return emptyList()
        val saved = prefs.getString(key, null) ?: return emptyList()
        return runCatching {
            val listType = object : TypeToken<List<Meta>>() {}.type
            gson.fromJson<List<Meta>>(saved, listType)
                ?.filter { it.id.isNotBlank() && it.name.isNotBlank() }
                .orEmpty()
        }.getOrElse { emptyList() }
    }

    fun save(list: List<Meta>, profile: UserProfile?) {
        val key = key(profile) ?: return
        prefs.edit().putString(key, gson.toJson(list)).apply()
    }

    private fun key(profile: UserProfile?): String? = profile?.id?.takeIf { it.isNotBlank() }?.let { "history_$it" }
}

@Singleton
class ForgottenContinueWatchingStore @Inject constructor(@ApplicationContext context: Context) {
    private val prefs = context.getSharedPreferences("fluxa_search", Context.MODE_PRIVATE)

    fun load(profile: UserProfile?): Set<String> {
        val key = key(profile) ?: return emptySet()
        return prefs.getStringSet(key, emptySet()).orEmpty()
    }

    fun save(profile: UserProfile?, keys: Set<String>) {
        val key = key(profile) ?: return
        prefs.edit().putStringSet(key, keys).apply()
    }

    private fun key(profile: UserProfile?): String? = profile?.id?.takeIf { it.isNotBlank() }?.let { "forgotten_continue_$it" }
}

internal fun mergeSyncedProfile(gson: Gson, base: UserProfile, updated: UserProfile, current: UserProfile?): UserProfile {
    if (current == null || current.id != base.id || updated.id != base.id) return updated
    val baseJson = gson.toJsonTree(base).asJsonObject
    val updatedJson = gson.toJsonTree(updated).asJsonObject
    val mergedJson = gson.toJsonTree(current).asJsonObject
    for ((key, updatedValue) in updatedJson.entrySet()) {
        if (updatedValue != baseJson.get(key)) {
            mergedJson.add(key, updatedValue)
        }
    }
    return runCatching { gson.fromJson(mergedJson, UserProfile::class.java) }.getOrDefault(updated)
}

@Singleton
class HomeCategoryCache @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs = context.getSharedPreferences("fluxa_home_cache", Context.MODE_PRIVATE)

    fun load(profile: UserProfile?): List<HomeCategory> {
        val key = key(profile) ?: return emptyList()
        val saved = prefs.getString(key, null) ?: return emptyList()
        return runCatching {
            val listType = object : TypeToken<List<HomeCategory>>() {}.type
            gson.fromJson<List<HomeCategory>>(saved, listType).orEmpty()
        }.getOrElse { emptyList() }
    }

    fun save(profile: UserProfile?, list: List<HomeCategory>) {
        val key = key(profile) ?: return
        prefs.edit().putString(key, gson.toJson(list)).apply()
    }

    private fun key(profile: UserProfile?): String? = profile?.id?.takeIf { it.isNotBlank() }?.let { "home_cache_$it" }
}

@Singleton
class HomeBillboardCache @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs = context.getSharedPreferences("fluxa_home_billboard_cache", Context.MODE_PRIVATE)

    fun load(profile: UserProfile?): List<Meta> {
        val key = key(profile) ?: return emptyList()
        val saved = prefs.getString(key, null) ?: return emptyList()
        return runCatching {
            val listType = object : TypeToken<List<Meta>>() {}.type
            gson.fromJson<List<Meta>>(saved, listType).orEmpty()
        }.getOrElse { emptyList() }
    }

    fun save(profile: UserProfile?, pool: List<Meta>) {
        val key = key(profile) ?: return
        prefs.edit().putString(key, gson.toJson(pool)).apply()
    }

    private fun key(profile: UserProfile?): String? = profile?.id?.takeIf { it.isNotBlank() }?.let { "billboard_cache_$it" }
}
