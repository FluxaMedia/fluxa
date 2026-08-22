package com.fluxa.app.data.local

import com.fluxa.app.data.platform.PlatformSecureStore
import com.google.gson.Gson
import kotlinx.coroutines.runBlocking
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProfileCredentialStore @Inject constructor(
    private val secureStore: PlatformSecureStore,
    private val gson: Gson
) {
    fun hydrate(profile: UserProfile): UserProfile {
        val credentials = read(profile.id) ?: return profile
        val externalAccounts = profile.externalAccounts?.copy(
            traktAccessToken = credentials.traktAccessToken ?: profile.traktAccessToken,
            traktRefreshToken = credentials.traktRefreshToken ?: profile.traktRefreshToken,
            simklAccessToken = credentials.simklAccessToken ?: profile.simklAccessToken
        )
        return profile.copy(
            authKey = credentials.authKey ?: profile.authKey,
            pinHash = credentials.pinHash ?: profile.pinHash,
            nuvioAccessToken = credentials.nuvioAccessToken ?: profile.nuvioAccessToken,
            nuvioRefreshToken = credentials.nuvioRefreshToken ?: profile.nuvioRefreshToken,
            traktAccessToken = credentials.traktAccessToken ?: profile.traktAccessToken,
            traktRefreshToken = credentials.traktRefreshToken ?: profile.traktRefreshToken,
            simklAccessToken = credentials.simklAccessToken ?: profile.simklAccessToken,
            anilistAccessToken = credentials.anilistAccessToken ?: profile.anilistAccessToken,
            anilistRefreshToken = credentials.anilistRefreshToken ?: profile.anilistRefreshToken,
            tmdbApiKey = credentials.tmdbApiKey ?: profile.tmdbApiKey,
            mdblistApiKey = credentials.mdblistApiKey ?: profile.mdblistApiKey,
            introDbApiKey = credentials.introDbApiKey ?: profile.introDbApiKey,
            externalAccounts = externalAccounts
        )
    }

    fun store(profile: UserProfile) = runBlocking {
        val value = gson.toJson(ProfileCredentials.from(profile))
        secureStore.writeSecret(profile.id, value)
    }

    fun remove(profileId: String) = runBlocking {
        secureStore.removeSecret(profileId)
    }

    fun redact(profile: UserProfile): UserProfile = profile.copy(
        authKey = "",
        pinHash = null,
        nuvioAccessToken = null,
        nuvioRefreshToken = null,
        traktAccessToken = null,
        traktRefreshToken = null,
        simklAccessToken = null,
        anilistAccessToken = null,
        anilistRefreshToken = null,
        tmdbApiKey = null,
        mdblistApiKey = null,
        introDbApiKey = null,
        externalAccounts = profile.externalAccounts?.copy(
            traktAccessToken = null,
            traktRefreshToken = null,
            simklAccessToken = null
        )
    )

    fun hasLegacyCredentials(profile: UserProfile): Boolean = profile != redact(profile)

    private fun read(profileId: String): ProfileCredentials? = runCatching {
        runBlocking { secureStore.readSecret(profileId) }?.let { gson.fromJson(it, ProfileCredentials::class.java) }
    }.getOrNull()
}

private data class ProfileCredentials(
    val authKey: String? = null,
    val pinHash: String? = null,
    val nuvioAccessToken: String? = null,
    val nuvioRefreshToken: String? = null,
    val traktAccessToken: String? = null,
    val traktRefreshToken: String? = null,
    val simklAccessToken: String? = null,
    val anilistAccessToken: String? = null,
    val anilistRefreshToken: String? = null,
    val tmdbApiKey: String? = null,
    val mdblistApiKey: String? = null,
    val introDbApiKey: String? = null
) {
    companion object {
        fun from(profile: UserProfile) = ProfileCredentials(
            authKey = profile.authKey,
            pinHash = profile.pinHash,
            nuvioAccessToken = profile.nuvioAccessToken,
            nuvioRefreshToken = profile.nuvioRefreshToken,
            traktAccessToken = profile.traktAccessToken,
            traktRefreshToken = profile.traktRefreshToken,
            simklAccessToken = profile.simklAccessToken,
            anilistAccessToken = profile.anilistAccessToken,
            anilistRefreshToken = profile.anilistRefreshToken,
            tmdbApiKey = profile.tmdbApiKey,
            mdblistApiKey = profile.mdblistApiKey,
            introDbApiKey = profile.introDbApiKey
        )
    }
}
