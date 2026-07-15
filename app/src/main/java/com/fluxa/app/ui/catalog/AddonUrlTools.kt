package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.StremioAddonUrls

internal fun addonNameFromUrl(url: String): String {
    val host = runCatching { java.net.URI(url).host.orEmpty() }.getOrDefault("")
        .removePrefix("www.")
        .substringBefore(".")
    return when {
        url.contains("torrentio", ignoreCase = true) -> "Torrentio"
        url.contains("opensubtitles", ignoreCase = true) -> "OpenSubtitles"
        url.contains("cizgivedizi", ignoreCase = true) -> "Cizgi ve Dizi"
        host.isNotBlank() -> host.replace('-', ' ').replaceFirstChar {
            if (it.isLowerCase()) it.titlecase() else it.toString()
        }
        else -> "Stremio Addon"
    }
}

internal fun addonConfigUrl(url: String): String? {
    return StremioAddonUrls.normalizeManifestUrl(url)
        .substringBefore("/manifest.json")
        .takeIf { it.startsWith("http://") || it.startsWith("https://") }
}

internal fun normalizeAddonUrlForProfile(rawUrl: String): String {
    return FluxaCoreNative.normalizeManifestUrl(rawUrl)
}

internal fun addonUrlIdentity(rawUrl: String): String {
    return FluxaCoreNative.identity(rawUrl)
}
