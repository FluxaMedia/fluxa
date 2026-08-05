package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.AvatarPackRecord
import com.fluxa.app.data.local.AvatarRecord
import com.fluxa.app.data.remote.StremioService
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request

class ProfileAvatarPackRepository {
    private val httpClient get() = StremioService.sharedClient

    suspend fun discover(repositoryUrl: String): Result<List<AvatarPackRecord>> = withContext(Dispatchers.IO) {
        runCatching {
            val repositoryPlan = FluxaCoreNative.profileAvatarPackRepositoryPlan(
                JsonObject().apply { addProperty("repositoryUrl", repositoryUrl) }.toString()
            ) ?: return@runCatching emptyList()

            val repository = fetchJson(repositoryPlan.repositoryApiUrl)
            val discoveryPlan = FluxaCoreNative.profileAvatarPackDiscoveryPlan(
                JsonObject().apply {
                    addProperty("repositoryUrl", repositoryUrl)
                    add("repository", repository)
                }.toString()
            ) ?: return@runCatching emptyList()

            val tree = fetchJson(discoveryPlan.treeApiUrl)
            val catalog = FluxaCoreNative.profileAvatarPackCatalog(
                JsonObject().apply {
                    addProperty("repositoryUrl", repositoryUrl)
                    addProperty("reference", discoveryPlan.reference)
                    add("tree", tree)
                }.toString()
            ) ?: return@runCatching emptyList()

            catalog.categories.mapNotNull { category ->
                val pack = runCatching { fetchJson(category.manifestUrl) }.getOrNull() ?: return@mapNotNull null
                val parsed = FluxaCoreNative.profileAvatarPackParse(
                    JsonObject().apply {
                        addProperty("manifestUrl", category.manifestUrl)
                        add("pack", pack)
                    }.toString()
                ) ?: return@mapNotNull null
                if (parsed.avatars.isEmpty()) return@mapNotNull null
                AvatarPackRecord(
                    id = parsed.manifestUrl,
                    repositoryUrl = repositoryUrl,
                    title = parsed.title,
                    avatars = parsed.avatars.map { AvatarRecord(name = it.name, url = it.url) }
                )
            }
        }
    }

    private fun fetchJson(url: String) = httpClient.newCall(Request.Builder().url(url).build()).execute().use { response ->
        if (!response.isSuccessful) error("Request to $url failed with ${response.code}")
        JsonParser.parseString(response.body.string())
    }
}
