package com.fluxa.app.player

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

fun resolveAudioLanguagePreference(
    profile: UserProfile?,
    meta: Meta,
    deviceLanguage: String?
): String? {
    val isAnime = meta.genres?.any { it.contains("anime", true) } == true
    if (isAnime && profile?.safeAnimePreferJapaneseAudio == true) return "ja"
    return when (val preference = profile?.preferredAudioLanguage) {
        "original" -> meta.originalLanguage?.takeIf(String::isNotBlank)
        "device_language" -> deviceLanguage?.takeIf(String::isNotBlank)
        else -> preference
    }
}
