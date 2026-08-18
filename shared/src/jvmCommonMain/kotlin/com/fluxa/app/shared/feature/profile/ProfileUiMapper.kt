package com.fluxa.app.shared.feature.profile

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAccentColorArgb
import com.fluxa.app.data.local.safeLanguage

fun UserProfile.toProfileUiModel(): ProfileUiModel = ProfileUiModel(
    id = id,
    name = profileName?.takeIf { it.isNotBlank() } ?: email,
    avatarUrl = avatarUrl,
    language = safeLanguage,
    accentColorArgb = safeAccentColorArgb.toLong() and 0xffffffffL,
    hasPin = !pinHash.isNullOrBlank() || nuvioPinEnabled,
    nuvioPinEnabled = nuvioPinEnabled,
    nuvioProfileIndex = nuvioProfileIndex,
    biometricEnabled = biometricEnabled == true,
)
