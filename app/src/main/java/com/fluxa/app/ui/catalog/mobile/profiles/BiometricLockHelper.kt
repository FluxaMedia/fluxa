package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.fluxa.app.common.AppStrings

object BiometricLockHelper {

    fun isAvailable(context: Context): Boolean {
        val manager = BiometricManager.from(context)
        return manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_WEAK) ==
            BiometricManager.BIOMETRIC_SUCCESS
    }

    fun authenticate(
        activity: FragmentActivity,
        lang: String?,
        onSuccess: () -> Unit,
        onFailure: () -> Unit
    ) {
        val prompt = BiometricPrompt(
            activity,
            ContextCompat.getMainExecutor(activity),
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    onSuccess()
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
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_WEAK)
            .build()

        prompt.authenticate(info)
    }
}
