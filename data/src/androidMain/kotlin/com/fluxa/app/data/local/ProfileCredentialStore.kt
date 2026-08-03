package com.fluxa.app.data.local

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import com.google.gson.Gson
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import android.util.Base64
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProfileCredentialStore @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val preferences = context.getSharedPreferences("fluxa_profile_credentials", Context.MODE_PRIVATE)

    fun hydrate(profile: UserProfile): UserProfile {
        val credentials = read(profile.id) ?: return profile
        val externalAccounts = profile.externalAccounts?.copy(
            traktAccessToken = credentials.traktAccessToken ?: profile.externalAccounts.traktAccessToken,
            traktRefreshToken = credentials.traktRefreshToken ?: profile.externalAccounts.traktRefreshToken,
            simklAccessToken = credentials.simklAccessToken ?: profile.externalAccounts.simklAccessToken
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

    fun store(profile: UserProfile) {
        val value = gson.toJson(ProfileCredentials.from(profile))
        preferences.edit().putString(profile.id, encrypt(value)).apply()
    }

    fun remove(profileId: String) {
        preferences.edit().remove(profileId).apply()
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
        preferences.getString(profileId, null)?.let(::decrypt)?.let { gson.fromJson(it, ProfileCredentials::class.java) }
    }.getOrNull()

    private fun encrypt(value: String): String {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key())
        return Base64.encodeToString(cipher.iv + cipher.doFinal(value.encodeToByteArray()), Base64.NO_WRAP)
    }

    private fun decrypt(value: String): String {
        val bytes = Base64.decode(value, Base64.NO_WRAP)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
        return bytes.copyOfRange(12, bytes.size).toString(Charsets.UTF_8)
    }

    private fun key(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .build())
        }.generateKey()
    }

    private companion object { const val KEY_ALIAS = "fluxa.profile.credentials.v1" }
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
            traktAccessToken = profile.traktAccessToken ?: profile.externalAccounts?.traktAccessToken,
            traktRefreshToken = profile.traktRefreshToken ?: profile.externalAccounts?.traktRefreshToken,
            simklAccessToken = profile.simklAccessToken ?: profile.externalAccounts?.simklAccessToken,
            anilistAccessToken = profile.anilistAccessToken,
            anilistRefreshToken = profile.anilistRefreshToken,
            tmdbApiKey = profile.tmdbApiKey,
            mdblistApiKey = profile.mdblistApiKey,
            introDbApiKey = profile.introDbApiKey
        )
    }
}
