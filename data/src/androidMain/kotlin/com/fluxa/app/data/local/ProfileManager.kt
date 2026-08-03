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
    private val gson: Gson,
    private val credentialStore: ProfileCredentialStore
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

    @WorkerThread
    fun updateProfile(profileId: String, transform: (UserProfile) -> UserProfile): UserProfile? = synchronized(profilesLock) {
        val current = getProfiles().firstOrNull { it.id == profileId } ?: return@synchronized null
        transform(current).also(::saveProfile)
    }

    private fun saveProfileInternal(profile: UserProfile, mergeMirroredAddons: Boolean) {
        val sanitizedProfile = sanitizeProfile(profile, mergeMirroredAddons)
        synchronized(profilesLock) {
            credentialStore.store(sanitizedProfile)
            val persistedProfile = credentialStore.redact(sanitizedProfile)
            val profiles = getProfiles().map(credentialStore::redact).toMutableList()
            val existingIndex = profiles.indexOfFirst {
                it.id == persistedProfile.id ||
                    (it.email == sanitizedProfile.email &&
                        it.nuvioUserId.isNullOrBlank() && persistedProfile.nuvioUserId.isNullOrBlank())
            }
            if (existingIndex != -1) {
                profiles[existingIndex] = persistedProfile
            } else {
                profiles.add(persistedProfile)
            }
            val json = gson.toJson(profiles)
            prefs.edit()
                .putString("profiles_list", json)
                .putStringSet(localAddonsKey(persistedProfile), persistedProfile.safeInstalledLocalAddons.toSet())
                .apply()
            cachedProfiles = profiles.map(credentialStore::hydrate)
        }
    }

    @WorkerThread
    fun recordExternalSyncFailure(profileId: String, provider: String) {
        updateProfile(profileId) { profile ->
            val current = profile.externalSyncFailedProviders.orEmpty()
            if (provider in current) profile else profile.copy(externalSyncFailedProviders = current + provider)
        }
    }

    @WorkerThread
    fun clearExternalSyncFailure(profileId: String, provider: String) {
        updateProfile(profileId) { profile ->
            val current = profile.externalSyncFailedProviders.orEmpty()
            if (provider !in current) profile else profile.copy(externalSyncFailedProviders = current - provider)
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
        val hasLegacyCredentials = list.any(credentialStore::hasLegacyCredentials)
        val hydrated = list.map { profile ->
            val sanitized = sanitizeProfile(profile, mergeMirroredAddons = true)
            if (credentialStore.hasLegacyCredentials(sanitized)) credentialStore.store(sanitized)
            credentialStore.hydrate(credentialStore.redact(sanitized))
        }
        if (hasLegacyCredentials) {
            prefs.edit().putString("profiles_list", gson.toJson(hydrated.map(credentialStore::redact))).apply()
        }
        return hydrated
            .distinctBy { profile ->
                when {
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
            getProfiles().filter { it.email == email }.forEach { credentialStore.remove(it.id) }
            val json = gson.toJson(profiles)
            prefs.edit().putString("profiles_list", json).apply()
            cachedProfiles = profiles
        }
    }

    @WorkerThread
    fun deleteProfileById(id: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.id == id }
            credentialStore.remove(id)
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

    fun canAttemptPin(profileId: String, nowMillis: Long = System.currentTimeMillis()): Boolean {
        val value = prefs.getString(pinAttemptKey(profileId), null) ?: return true
        val parts = value.split(':')
        val failures = parts.getOrNull(0)?.toIntOrNull() ?: return true
        val lastFailure = parts.getOrNull(1)?.toLongOrNull() ?: return true
        val delay = if (failures >= 5) 300_000L else minOf(30_000L, 1_000L shl (failures - 1).coerceAtLeast(0))
        return nowMillis - lastFailure >= delay
    }

    fun recordPinFailure(profileId: String, nowMillis: Long = System.currentTimeMillis()) {
        val previous = prefs.getString(pinAttemptKey(profileId), null)?.substringBefore(':')?.toIntOrNull() ?: 0
        prefs.edit().putString(pinAttemptKey(profileId), "${minOf(previous + 1, 5)}:$nowMillis").apply()
    }

    fun clearPinFailures(profileId: String) {
        prefs.edit().remove(pinAttemptKey(profileId)).apply()
    }

    fun registerOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.registerOnSharedPreferenceChangeListener(listener)
    }

    fun unregisterOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.unregisterOnSharedPreferenceChangeListener(listener)
    }

    private fun pinAttemptKey(profileId: String): String = "pin_attempt_$profileId"
}
