package com.fluxa.app.data.local

import android.content.Context
import android.content.SharedPreferences
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import androidx.annotation.WorkerThread
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProfileManager @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs: SharedPreferences = context.getSharedPreferences("fluxa_profiles", Context.MODE_PRIVATE)
    private val profilesLock = Any()
    @Volatile private var cachedProfiles: List<UserProfile>? = null

    @WorkerThread
    fun saveProfile(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = true)
    }

    @WorkerThread
    fun saveProfileReplacingLocalAddons(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = false)
    }

    private fun saveProfileInternal(profile: UserProfile, mergeMirroredAddons: Boolean) {
        val sanitizedProfile = sanitizeProfile(profile, mergeMirroredAddons)
        // The whole read-modify-write must be one atomic unit, not just the cache swap —
        // otherwise two concurrent callers (e.g. a WorkManager worker and a coroutine
        // completion callback) can each read a stale list and clobber each other's write.
        synchronized(profilesLock) {
            val profiles = getProfiles().toMutableList()
            val existingIndex = profiles.indexOfFirst {
                it.id == sanitizedProfile.id ||
                    (it.email == sanitizedProfile.email && !it.isGuest &&
                        it.nuvioUserId.isNullOrBlank() && sanitizedProfile.nuvioUserId.isNullOrBlank())
            }
            if (existingIndex != -1) {
                profiles[existingIndex] = sanitizedProfile
            } else {
                profiles.add(sanitizedProfile)
            }
            val json = gson.toJson(profiles)
            prefs.edit()
                .putString("profiles_list", json)
                .putStringSet(localAddonsKey(sanitizedProfile), sanitizedProfile.safeInstalledLocalAddons.toSet())
                .apply()
            cachedProfiles = profiles
        }
    }

    fun getProfiles(): List<UserProfile> {
        cachedProfiles?.let { return it }
        return synchronized(profilesLock) {
            cachedProfiles ?: loadProfiles().also { cachedProfiles = it }
        }
    }

    @WorkerThread
    private fun loadProfiles(): List<UserProfile> {
        val json = prefs.getString("profiles_list", null)
        if (json == null) {
            return emptyList()
        }
        val type = object : TypeToken<List<UserProfile>>() {}.type
        val list: List<UserProfile> = gson.fromJson(json, type)
        return list
            .map { sanitizeProfile(it, mergeMirroredAddons = true) }
            .distinctBy { profile ->
                when {
                    profile.isGuest -> profile.id
                    !profile.nuvioUserId.isNullOrBlank() -> "nuvio:${profile.nuvioUserId}:${profile.nuvioProfileIndex ?: 1}"
                    else -> profile.email
                }
            }
    }

    private fun sanitizeProfile(profile: UserProfile, mergeMirroredAddons: Boolean): UserProfile {
        val mirroredAddons = if (mergeMirroredAddons) {
            prefs.getStringSet(localAddonsKey(profile), emptySet()).orEmpty()
        } else {
            emptySet()
        }
        return FluxaCoreNative.sanitizeProfile(
            profile = profile,
            mirroredAddons = mirroredAddons,
            mergeMirroredAddons = mergeMirroredAddons,
            type = UserProfile::class.java
        ) ?: profile.withStructuredSettings()
    }

    private fun localAddonsKey(profile: UserProfile): String {
        return FluxaCoreNative.profileLocalAddonsKey(profile)
    }

    @WorkerThread
    fun deleteProfile(email: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.email == email }
            val json = gson.toJson(profiles)
            prefs.edit().putString("profiles_list", json).apply()
            cachedProfiles = profiles
        }
    }

    @WorkerThread
    fun deleteProfileById(id: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.id == id }
            val json = gson.toJson(profiles)
            prefs.edit().putString("profiles_list", json).apply()
            cachedProfiles = profiles
        }
    }

    fun getLastActiveProfileId(): String? {
        return prefs.getString("last_active_profile_id", null)
    }

    fun setLastActiveProfile(profile: UserProfile?) {
        val editor = prefs.edit()
        if (profile == null) {
            editor.remove("last_active_profile_id")
        } else {
            editor.putString("last_active_profile_id", profile.id)
        }
        editor.apply()
    }

    fun registerOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.registerOnSharedPreferenceChangeListener(listener)
    }

    fun unregisterOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.unregisterOnSharedPreferenceChangeListener(listener)
    }
}
