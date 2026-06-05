package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import okhttp3.Request
import org.jsoup.Jsoup
import java.io.IOException

internal fun parseStremioAddonSearchHtml(
    html: String,
    resolveManifestUrl: (String) -> String?
): List<CommunityAddon> {
    val document = Jsoup.parse(html, "https://stremio-addons.net")
    return document.select("a.block[href^=/addons/]")
        .asSequence()
        .take(10)
        .mapNotNull { card ->
            val path = card.attr("href").takeIf { it.startsWith("/addons/") } ?: return@mapNotNull null
            val manifestUrl = resolveManifestUrl(path) ?: return@mapNotNull null
            val logo = card.selectFirst("img")?.let { image ->
                image.absUrl("src").ifBlank { image.attr("src") }.ifBlank { image.absUrl("data-src") }.ifBlank { image.attr("data-src") }
            }?.takeIf { it.isNotBlank() }
            CommunityAddon(
                name = card.selectFirst("h3")?.text().orEmpty().trim(),
                description = card.selectFirst("p")?.text().orEmpty().trim(),
                url = manifestUrl,
                logoUrl = logo?.let(FluxaCoreNative::preferHttpsAssetUrl)
            )
        }
        .filter { it.name.isNotBlank() }
        .toList()
}

internal fun resolveAddonManifestUrl(path: String): String? {
    val detailHtml = runCatching { fetchUrl("https://stremio-addons.net$path") }.getOrNull() ?: return null
    val detailText = Jsoup.parse(detailHtml, "https://stremio-addons.net").html()
    return FluxaCoreNative.extractAddonManifestUrl(detailText)
}

internal fun fetchUrl(url: String): String {
    val request = Request.Builder().url(url).header("User-Agent", com.fluxa.app.BuildConfig.APPLICATION_ID).build()
    return addonStoreClient.newCall(request).execute().use { response ->
        if (!response.isSuccessful) throw IOException("HTTP ${response.code} for $url")
        response.body.string()
    }
}
