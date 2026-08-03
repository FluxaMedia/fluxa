package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.fluxa.app.common.AppStrings
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

object BiometricLockHelper {

    fun isAvailable(context: Context): Boolean {
        val manager = BiometricManager.from(context)
        return manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) ==
            BiometricManager.BIOMETRIC_SUCCESS
    }

    fun authenticate(
        activity: FragmentActivity,
        lang: String?,
        profileId: String,
        onSuccess: () -> Unit,
        onFailure: () -> Unit
    ) {
        val cipher = BiometricProfileKeyStore.authenticationCipher(activity, profileId) ?: run {
            onFailure()
            return
        }
        val prompt = BiometricPrompt(
            activity,
            ContextCompat.getMainExecutor(activity),
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    val authenticated = result.cryptoObject?.cipher?.let { cipher ->
                        BiometricProfileKeyStore.verify(activity, profileId, cipher)
                    } == true
                    if (authenticated) onSuccess() else onFailure()
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    onFailure()
                }

                override fun onAuthenticationFailed() {
                    onFailure()
                }
            }
        )

        val info = BiometricPrompt.PromptInfo.Builder()
            .setTitle(AppStrings.t(lang, "profiles.biometric_prompt_title"))
            .setSubtitle(AppStrings.t(lang, "profiles.biometric_prompt_subtitle"))
            .setNegativeButtonText(AppStrings.t(lang, "profiles.use_pin"))
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
            .build()

        prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
    }
}

private object BiometricProfileKeyStore {
    private const val PreferencesName = "fluxa_biometric_profiles"
    private const val KeyPrefix = "fluxa.profile.biometric."

    fun authenticationCipher(context: Context, profileId: String): Cipher? = runCatching {
        val encryptedVerifier = encryptedVerifier(context, profileId)
        val bytes = Base64.decode(encryptedVerifier, Base64.NO_WRAP)
        Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.DECRYPT_MODE, key(profileId), GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
        }
    }.getOrNull()

    fun verify(context: Context, profileId: String, cipher: Cipher): Boolean = runCatching {
        val bytes = Base64.decode(encryptedVerifier(context, profileId), Base64.NO_WRAP)
        cipher.doFinal(bytes.copyOfRange(12, bytes.size)).isNotEmpty()
    }.getOrDefault(false)

    private fun encryptedVerifier(context: Context, profileId: String): String {
        val preferences = context.applicationContext.getSharedPreferences(PreferencesName, Context.MODE_PRIVATE)
        preferences.getString(profileId, null)?.let { return it }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.ENCRYPT_MODE, key(profileId))
        }
        val value = Base64.encodeToString(cipher.iv + cipher.doFinal(ByteArray(32).also(SecureRandom()::nextBytes)), Base64.NO_WRAP)
        preferences.edit().putString(profileId, value).apply()
        return value
    }

    private fun key(profileId: String): SecretKey {
        val alias = KeyPrefix + profileId
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(alias, null) as? SecretKey)?.let { return it }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(
                KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                    .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                    .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                    .setUserAuthenticationRequired(true)
                    .setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG)
                    .setInvalidatedByBiometricEnrollment(true)
                    .build()
            )
        }.generateKey()
    }
}
