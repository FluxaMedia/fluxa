package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import org.junit.Assert.assertEquals
import org.junit.Test

class AddonStoreSearchParserTest {

    @Test
    fun parserExtractsAddonCardsWithDecodedText() {
        val html = """
            <html>
              <body>
                <a class="block" href="/addons/example">
                  <img loading="lazy" src="https://cdn.example/logo.png">
                  <h3>Example &amp; Addon</h3>
                  <p>Fast &lt;strong&gt;streams&lt;/strong&gt;</p>
                </a>
              </body>
            </html>
        """.trimIndent()

        val addons = parseStremioAddonSearchHtml(html) { path ->
            assertEquals("/addons/example", path)
            "https://addon.example/manifest.json"
        }

        assertEquals(1, addons.size)
        assertEquals("Example & Addon", addons.single().name)
        assertEquals("Fast <strong>streams</strong>", addons.single().description)
        assertEquals("https://addon.example/manifest.json", addons.single().url)
        assertEquals("https://cdn.example/logo.png", addons.single().logoUrl)
    }

    @Test
    fun parserUpgradesHttpLogoUrls() {
        val html = """
            <a class="block" href="/addons/hdhub">
              <img src="http://hdhub.thevolecitor.qzz.io/logo.png">
              <h3>HdHub</h3>
              <p>Streams</p>
            </a>
        """.trimIndent()

        val addons = parseStremioAddonSearchHtml(html) { "https://hdhub.thevolecitor.qzz.io/manifest.json" }

        assertEquals("https://hdhub.thevolecitor.qzz.io/logo.png", addons.single().logoUrl)
    }

    @Test
    fun parserSkipsCardsWithoutManifest() {
        val html = """
            <a class="block" href="/addons/missing">
              <h3>Missing Manifest</h3>
              <p>No manifest URL on detail page.</p>
            </a>
        """.trimIndent()

        val addons = parseStremioAddonSearchHtml(html) { null }

        assertEquals(emptyList<CommunityAddon>(), addons)
    }

    @Test
    fun nativeAddonStorePolicyDetectsAndNormalizesInputs() {
        assertEquals("unknown", FluxaCoreNative.addonStoreInputType(" "))
        assertEquals("stremio_manifest", FluxaCoreNative.addonStoreInputType("https://addon.example/manifest.json"))
        assertEquals("cloudstream_repo", FluxaCoreNative.addonStoreInputType("cloudstreamrepo://example.com/repo.json"))
        assertEquals("search_query", FluxaCoreNative.addonStoreInputType("torrentio"))
        assertEquals(
            "https://example.com/repo.json",
            FluxaCoreNative.normalizeCloudstreamRepoUrl("cloudstream://example.com/repo.json")
        )
    }

    @Test
    fun nativeAddonStoreSearchPolicyOwnsCacheAndUrlPlanning() {
        val cached = FluxaCoreNative.addonStoreSearchPolicy(
            query = "Game of Thrones",
            nowMillis = 2_000,
            cachedAtMillis = 1_500,
            ttlMillis = 1_000
        )

        assertEquals("game of thrones", cached.normalizedQuery)
        assertEquals("https://stremio-addons.net/addons?query=game+of+thrones", cached.url)
        assertEquals(true, cached.useCache)
        assertEquals(false, cached.shouldFetch)
    }

    @Test
    fun nativeManifestExtractorUnescapesDetailPageUrl() {
        val detail = """<script>"https://addon.example\/manifest.json?x=1\u0026y=2"</script>"""

        assertEquals(
            "https://addon.example/manifest.json?x=1&y=2",
            FluxaCoreNative.extractAddonManifestUrl(detail)
        )
    }
}
