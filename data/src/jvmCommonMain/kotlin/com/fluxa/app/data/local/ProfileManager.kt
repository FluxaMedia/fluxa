package com.fluxa.app.data.local

import com.fluxa.app.common.epochMillisNow
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.runBlocking
import java.util.concurrent.CopyOnWriteArrayList
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton

@Singleton
class ProfileManager @Inject constructor(
    @param:Named("ProfilePrefs") private val prefsStore: PlatformKeyValueStore,
    private val gson: Gson,
    private val credentialStore: ProfileCredentialStore
) {
    private val profilesLock = Any()
    @Volatile private var cachedProfiles: List<UserProfile>? = null
    private val changeListeners = CopyOnWriteArrayList<() -> Unit>()
    private val stringSetType = object : TypeToken<Set<String>>() {}.type

    private fun prefsGet(key: String): String? = runBlocking { prefsStore.read(key) }

    private fun prefsPut(key: String, value: String) = runBlocking { prefsStore.write(key, value) }

    private fun prefsRemove(key: String) = runBlocking { prefsStore.remove(key) }

    private fun notifyChanged() {
        changeListeners.forEach { it() }
    }

    fun addChangeListener(listener: () -> Unit) {
        changeListeners.add(listener)
    }

    fun removeChangeListener(listener: () -> Unit) {
        changeListeners.remove(listener)
    }

    fun saveProfile(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = true)
    }

    fun saveProfileReplacingLocalAddons(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = false)
    }

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
            prefsPut("profiles_list", gson.toJson(profiles))
            prefsPut(localAddonsKey(persistedProfile), gson.toJson(persistedProfile.safeInstalledLocalAddons.toSet()))
            cachedProfiles = profiles.map(credentialStore::hydrate)
        }
        notifyChanged()
    }

    fun recordExternalSyncFailure(profileId: String, provider: String) {
        updateProfile(profileId) { profile ->
            val current = profile.externalSyncFailedProviders.orEmpty()
            if (provider in current) profile else profile.copy(externalSyncFailedProviders = current + provider)
        }
    }

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

    private fun loadProfiles(): List<UserProfile> {
        val json = prefsGet("profiles_list") ?: return emptyList()
        val type = object : TypeToken<List<UserProfile>>() {}.type
        val list: List<UserProfile> = gson.fromJson(json, type)
        val hasLegacyCredentials = list.any(credentialStore::hasLegacyCredentials)
        val hydrated = list.map { profile ->
            val sanitized = sanitizeProfile(profile, mergeMirroredAddons = true)
            if (credentialStore.hasLegacyCredentials(sanitized)) credentialStore.store(sanitized)
            credentialStore.hydrate(credentialStore.redact(sanitized))
        }
        if (hasLegacyCredentials) {
            prefsPut("profiles_list", gson.toJson(hydrated.map(credentialStore::redact)))
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
            prefsGet(localAddonsKey(profile))?.let { gson.fromJson<Set<String>>(it, stringSetType) }.orEmpty()
        } else {
            emptySet()
        }
        val sanitized = FluxaCoreNative.sanitizeProfile(
            profile = profile,
            mirroredAddons = mirroredAddons,
            mergeMirroredAddons = mergeMirroredAddons,
            type = UserProfile::class.java
        ) ?: profile.withStructuredSettings()
        return sanitized
            .withKotlinOwnedProviderStateFrom(profile)
            .withPinnedLegacyStremioIdentity()
    }

    private fun localAddonsKey(profile: UserProfile): String {
        return FluxaCoreNative.profileLocalAddonsKey(profile)
    }

    fun deleteProfile(email: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.email == email }
            getProfiles().filter { it.email == email }.forEach { credentialStore.remove(it.id) }
            prefsPut("profiles_list", gson.toJson(profiles))
            cachedProfiles = profiles
        }
        notifyChanged()
    }

    fun deleteProfileById(id: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.id == id }
            credentialStore.remove(id)
            prefsPut("profiles_list", gson.toJson(profiles))
            cachedProfiles = profiles
        }
        notifyChanged()
    }

    fun getLastActiveProfileId(): String? {
        return prefsGet("last_active_profile_id")
    }

    fun isRememberLastProfileEnabled(): Boolean {
        return prefsGet("remember_last_profile")?.toBooleanStrictOrNull() ?: true
    }

    fun setRememberLastProfile(enabled: Boolean) {
        prefsPut("remember_last_profile", enabled.toString())
        notifyChanged()
    }

    fun setLastActiveProfile(profile: UserProfile?) {
        if (profile == null) {
            prefsRemove("last_active_profile_id")
        } else {
            prefsPut("last_active_profile_id", profile.id)
        }
        notifyChanged()
    }

    fun canAttemptPin(profileId: String, nowMillis: Long = epochMillisNow()): Boolean {
        val value = prefsGet(pinAttemptKey(profileId)) ?: return true
        val parts = value.split(':')
        val failures = parts.getOrNull(0)?.toIntOrNull() ?: return true
        val lastFailure = parts.getOrNull(1)?.toLongOrNull() ?: return true
        val delay = if (failures >= 5) 300_000L else minOf(30_000L, 1_000L shl (failures - 1).coerceAtLeast(0))
        return nowMillis - lastFailure >= delay
    }

    fun recordPinFailure(profileId: String, nowMillis: Long = epochMillisNow()) {
        val previous = prefsGet(pinAttemptKey(profileId))?.substringBefore(':')?.toIntOrNull() ?: 0
        prefsPut(pinAttemptKey(profileId), "${minOf(previous + 1, 5)}:$nowMillis")
        notifyChanged()
    }

    fun clearPinFailures(profileId: String) {
        prefsRemove(pinAttemptKey(profileId))
        notifyChanged()
    }

    fun getNuvioPinCache(profileId: String): String? = prefsGet("nuvio_pin_cache_$profileId")

    fun saveNuvioPinCache(profileId: String, payload: String) = prefsPut("nuvio_pin_cache_$profileId", payload)

    fun clearNuvioPinCache(profileId: String) = prefsRemove("nuvio_pin_cache_$profileId")

    private fun pinAttemptKey(profileId: String): String = "pin_attempt_$profileId"
}

private fun UserProfile.withKotlinOwnedProviderStateFrom(source: UserProfile): UserProfile = copy(
    stremioUserId = source.stremioUserId,
    stremioEmail = source.stremioEmail,
    providerSyncTimestamps = source.providerSyncTimestamps,
)

private fun UserProfile.withPinnedLegacyStremioIdentity(): UserProfile {
    if (authKey.isBlank()) return this
    if (!stremioUserId.isNullOrBlank() || !stremioEmail.isNullOrBlank()) return this
    return copy(stremioEmail = email.takeIf(String::isNotBlank))
}
