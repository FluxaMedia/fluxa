package com.fluxa.app.data.repository

import com.fluxa.app.data.local.UserProfile

/** Stable per-account/per-Nuvio-profile scope for persisted delta cursors and raw cache. */
internal fun UserProfile.nuvioSyncScope(): String? {
    val profileIndex = nuvioProfileIndex ?: return null
    val accountIdentity = nuvioUserId?.trim()?.takeIf(String::isNotBlank)
        ?: nuvioEmail?.trim()?.takeIf(String::isNotBlank)
        ?: id
    return "$accountIdentity#$profileIndex"
}
