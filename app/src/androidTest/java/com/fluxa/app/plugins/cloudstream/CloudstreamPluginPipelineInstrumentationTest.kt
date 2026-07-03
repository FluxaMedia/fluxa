package com.fluxa.app.plugins.cloudstream

import android.app.Application
import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.lagradost.cloudstream3.AcraApplication
import com.lagradost.cloudstream3.AnimeLoadResponse
import com.lagradost.cloudstream3.Episode
import com.lagradost.cloudstream3.LiveStreamLoadResponse
import com.lagradost.cloudstream3.LoadResponse
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.MainPageRequest
import com.lagradost.cloudstream3.MovieLoadResponse
import com.lagradost.cloudstream3.SearchResponse
import com.lagradost.cloudstream3.TvSeriesLoadResponse
import com.lagradost.cloudstream3.utils.ExtractorLink
import java.io.File
import java.security.MessageDigest
import java.util.zip.ZipFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class CloudstreamPluginPipelineInstrumentationTest {
    private val loadedIds = mutableListOf<String>()
    private lateinit var loader: ExternalExtensionLoader

    @After
    fun tearDown() {
        if (::loader.isInitialized) {
            loadedIds.forEach { id ->
                loader.unloadExtension(id)
                loader.deleteExtensionFile(id)
            }
        }
    }

    @Test
    fun phisherRepoPluginCatalogAndStreamPipelineWorksOnDevice() = runBlocking {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        AcraApplication.init(context.applicationContext as Application)
        loader = ExternalExtensionLoader(context)

        val repoUrl = "https://raw.githubusercontent.com/phisher98/cloudstream-extensions-phisher/refs/heads/builds/repo.json"
        val repoResult = ExternalRepoParser().fetchRepository(repoUrl)
        assertTrue("Phisher repo did not parse: $repoResult", repoResult is RepositoryResult.Success)
        val manifest = (repoResult as RepositoryResult.Success).manifest
        assertTrue("Expected plugin list from Phisher repo", manifest.plugins.isNotEmpty())

        val activeByName = manifest.plugins
            .filter { it.isActive && it.url.endsWith(".cs3") }
            .associateBy { it.internalName }
        assertTrue("Expected active .cs3 plugins", activeByName.isNotEmpty())

        val candidateNames = listOf(
            "YTS",
            "CloudPlay",
            "PublicSportsIPTV",
            "ToonTales",
            "AllWish",
            "AnimePahe"
        )

        var loadedAny = false
        var catalogAny = false
        var streamAny = false
        val failures = mutableListOf<String>()

        for (name in candidateNames) {
            val plugin = activeByName[name] ?: continue
            val installId = "instrument-${name.lowercase()}"
            loadedIds += installId

            try {
                val file = loader.downloadExtension(installId, plugin.url)
                assertNotNull("Download returned null for $name", file)
                file!!
                verifyPackage(file, plugin)

                val apis = loader.loadApisFromFile(installId, file)
                if (apis.isEmpty()) {
                    failures += "$name: no APIs loaded"
                    continue
                }
                loadedAny = true
                println("cs3-smoke-loaded $name apis=${apis.map { it.name }}")

                for (api in apis) {
                    val catalogResult = fetchCatalogCandidate(api)
                    if (catalogResult != null) {
                        catalogAny = true
                        println("cs3-smoke-catalog api=${api.name} result=${catalogResult.name}")
                        val links = loadLinksForSearchResponse(api, catalogResult)
                        if (links.isNotEmpty()) {
                            streamAny = true
                            println("cs3-smoke-stream api=${api.name} links=${links.size} first=${links.first().type}:${links.first().url.take(80)}")
                            break
                        }
                    }

                    val searchResult = searchCandidate(api)
                    if (searchResult != null) {
                        catalogAny = true
                        println("cs3-smoke-search api=${api.name} result=${searchResult.name}")
                        val links = loadLinksForSearchResponse(api, searchResult)
                        if (links.isNotEmpty()) {
                            streamAny = true
                            println("cs3-smoke-stream api=${api.name} links=${links.size} first=${links.first().type}:${links.first().url.take(80)}")
                            break
                        }
                    }
                }

                if (streamAny) break
            } catch (t: Throwable) {
                failures += "$name: ${t.javaClass.simpleName}: ${t.message}"
            } finally {
                loader.unloadExtension(installId)
                val deleted = loader.deleteExtensionFile(installId)
                println("cs3-smoke-remove $name deleted=$deleted")
            }
        }

        assertTrue("No real plugin loaded. Failures: $failures", loadedAny)
        assertTrue("No catalog/search content returned. Failures: $failures", catalogAny)
        assertTrue("No stream links returned. Failures: $failures", streamAny)
    }

    private fun verifyPackage(file: File, plugin: PluginInfo) {
        assertTrue("Downloaded plugin file missing: ${file.absolutePath}", file.exists())
        plugin.sha256?.let { expected ->
            assertEquals("Checksum mismatch for ${plugin.internalName}", expected.lowercase(), sha256(file))
        }
        ZipFile(file).use { zip ->
            assertNotNull("manifest.json missing for ${plugin.internalName}", zip.getEntry("manifest.json"))
            assertNotNull("classes.dex missing for ${plugin.internalName}", zip.getEntry("classes.dex"))
        }
    }

    private suspend fun fetchCatalogCandidate(api: MainAPI): SearchResponse? {
        if (!api.hasMainPage) return null
        return withTimeoutOrNull(20_000) {
            for (page in api.mainPage) {
                val result = runCatching {
                    api.getMainPage(
                        1,
                        MainPageRequest(page.name, page.data, page.horizontalImages)
                    )?.items?.flatMap { it.list }?.firstOrNull()
                }.getOrNull()
                if (result != null) return@withTimeoutOrNull result
            }
            null
        }
    }

    private suspend fun searchCandidate(api: MainAPI): SearchResponse? {
        val queries = listOf("Inception", "Naruto", "One Piece", "news")
        return withTimeoutOrNull(20_000) {
            for (query in queries) {
                val result = runCatching { api.search(query, 1)?.items?.firstOrNull() }.getOrNull()
                    ?: runCatching { api.search(query)?.firstOrNull() }.getOrNull()
                if (result != null) return@withTimeoutOrNull result
            }
            null
        }
    }

    private suspend fun loadLinksForSearchResponse(api: MainAPI, result: SearchResponse): List<ExtractorLink> {
        val loadResponse = withTimeoutOrNull(30_000) {
            runCatching { api.load(result.url) }.getOrNull()
        } ?: return emptyList()
        val data = extractPlayableData(loadResponse) ?: return emptyList()
        val links = mutableListOf<ExtractorLink>()
        withTimeoutOrNull(60_000) {
            runCatching {
                api.loadLinks(
                    data = data,
                    isCasting = false,
                    subtitleCallback = {},
                    callback = { link -> links += link }
                )
            }
        }
        return links.filter { it.url.isNotBlank() && it.url != "null" && it.url != "error" }
    }

    private fun extractPlayableData(response: LoadResponse): String? = when (response) {
        is MovieLoadResponse -> response.dataUrl
        is LiveStreamLoadResponse -> response.dataUrl
        is TvSeriesLoadResponse -> response.episodes.firstPlayableEpisode()?.data
        is AnimeLoadResponse -> response.episodes.values.flatten().firstPlayableEpisode()?.data
        else -> null
    }?.takeIf { it.isNotBlank() && it != "null" }

    private fun List<Episode>.firstPlayableEpisode(): Episode? =
        firstOrNull { it.data.isNotBlank() && it.data != "null" }

    private fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
            while (true) {
                val read = input.read(buffer)
                if (read <= 0) break
                digest.update(buffer, 0, read)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }
}
