package com.fluxa.app.ui

import android.content.Context
import android.util.Base64
import com.google.gson.Gson
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

fun legacyProfilesJson(context: Context): String {
    val prefs = context.getSharedPreferences("fluxa_profiles", Context.MODE_PRIVATE).all.mapValues { (_, value) ->
        when (value) {
            is Set<*> -> Gson().toJson(value.filterIsInstance<String>())
            else -> value?.toString()
        }
    }
    val key = runCatching {
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }.getKey("fluxa.profile.credentials.v1", null) as? SecretKey
    }.getOrNull()
    val credentials = if (key == null) emptyMap() else {
        context.getSharedPreferences("fluxa_profile_credentials", Context.MODE_PRIVATE).all.mapNotNull { (id, value) ->
            val bytes = runCatching { Base64.decode(value as String, Base64.NO_WRAP) }.getOrNull() ?: return@mapNotNull null
            runCatching {
                val cipher = Cipher.getInstance("AES/GCM/NoPadding")
                cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
                id to cipher.doFinal(bytes.copyOfRange(12, bytes.size)).toString(Charsets.UTF_8)
            }.getOrNull()
        }.toMap()
    }
    val picker = context.getSharedPreferences("fluxa_profile_picker", Context.MODE_PRIVATE).getString("picker_settings", null)
    return Gson().toJson(mapOf("prefs" to prefs, "credentials" to credentials, "picker" to picker))
}
