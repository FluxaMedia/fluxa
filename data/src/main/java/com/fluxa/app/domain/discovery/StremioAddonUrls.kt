package com.fluxa.app.domain.discovery

import com.fluxa.app.core.rust.FluxaCoreNative

object StremioAddonUrls {
    fun normalizeManifestUrl(rawUrl: String): String {
        return FluxaCoreNative.normalizeManifestUrl(rawUrl)
    }

    fun identity(rawUrl: String): String {
        return FluxaCoreNative.identity(rawUrl)
    }

    fun manifestCandidates(rawUrl: String): List<String> {
        return FluxaCoreNative.manifestCandidates(rawUrl)
    }

    fun baseUrl(rawUrl: String): String {
        return FluxaCoreNative.baseUrl(rawUrl)
    }

    fun preferHttpsAssetUrl(rawUrl: String?): String? {
        val trimmed = rawUrl?.trim()?.takeIf { it.isNotBlank() } ?: return null
        return FluxaCoreNative.preferHttpsAssetUrl(trimmed)
    }
}
