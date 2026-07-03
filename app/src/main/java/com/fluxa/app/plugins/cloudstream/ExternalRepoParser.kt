package com.fluxa.app.plugins.cloudstream

import android.util.Log
import com.fluxa.app.data.repository.HttpRequestSecurity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.concurrent.TimeUnit

private const val TAG = "ExternalRepoParser"
private const val MAX_RESPONSE_SIZE = 5 * 1024 * 1024L // 5MB max

/**
 * Parser for CloudStream3 external repositories
 * Handles JSON manifest parsing from repository URLs
 */
class ExternalRepoParser {
    
    private val httpClient = OkHttpClient.Builder()
        .addInterceptor { chain ->
            chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
        }
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .followRedirects(true)
        .build()

    /**
     * Normalize custom protocol schemes to https://
     * External repos often use schemes like "cloudstreamrepo://" or "stremio://"
     */
    private fun sanitizeScheme(url: String): String {
        val trimmed = url.trim()
        val schemeEnd = trimmed.indexOf("://")
        if (schemeEnd > 0) {
            val scheme = trimmed.substring(0, schemeEnd).lowercase()
            if (scheme != "http" && scheme != "https") {
                return "https://${trimmed.substring(schemeEnd + 3)}"
            }
        }
        return HttpRequestSecurity.preferHttps(trimmed)
    }

    /**
     * Fetch and parse a repository manifest
     */
    suspend fun fetchRepository(repoUrl: String): RepositoryResult = withContext(Dispatchers.IO) {
        try {
            val sanitizedUrl = sanitizeScheme(repoUrl)
            
            // Use URL as-is if it ends with .json, otherwise append manifest.json
            val manifestUrl = when {
                sanitizedUrl.endsWith(".json") -> sanitizedUrl
                sanitizedUrl.endsWith("manifest.json") -> sanitizedUrl
                sanitizedUrl.endsWith("/") -> "${sanitizedUrl}manifest.json"
                else -> "$sanitizedUrl/manifest.json"
            }
            
            Log.d(TAG, "Fetching repo: original=$repoUrl, manifestUrl=$manifestUrl")

            val request = Request.Builder()
                .url(manifestUrl)
                .build()

            httpClient.newCall(request).execute().use { response ->
                Log.d(TAG, "Repo response: ${response.code} for $manifestUrl")
                
                if (!response.isSuccessful) {
                    return@withContext RepositoryResult.Error(
                        "HTTP ${response.code}: ${response.message}"
                    )
                }

                val body = response.body.string()
                
                Log.d(TAG, "Repo body length: ${body.length} chars")

                // Check size
                if (body.length > MAX_RESPONSE_SIZE) {
                    return@withContext RepositoryResult.Error(
                        "Response too large: ${body.length} bytes"
                    )
                }

                parseRepositoryManifest(repoUrl, body, httpClient)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error fetching repository from $repoUrl", e)
            RepositoryResult.Error(e.message ?: "Unknown error")
        }
    }

    /**
     * Parse repository manifest JSON
     */
    private suspend fun parseRepositoryManifest(
        repoUrl: String, 
        jsonString: String, 
        httpClient: OkHttpClient
    ): RepositoryResult = withContext(Dispatchers.IO) {
        try {
            val json = JSONObject(jsonString)
            
            val name = json.optString("name", "Unknown Repository")
            val description = json.optString("description", "")
            val author = json.optString("author", "")
            val version = json.optInt("version", 1)
            val repoIconUrl = firstNonBlank(
                json.optNullableString("iconUrl"),
                json.optNullableString("icon"),
                json.optNullableString("logo")
            )?.let { resolveUrl(it, repoUrl) }
            
            val plugins = mutableListOf<PluginInfo>()
            
            // Check for pluginLists array (CloudStream3 standard format)
            json.optJSONArray("pluginLists")?.let { pluginListsArray ->
                Log.d(TAG, "Found pluginLists with ${pluginListsArray.length()} entries")
                
                for (i in 0 until pluginListsArray.length()) {
                    val pluginListUrl = pluginListsArray.getString(i)
                    Log.d(TAG, "Fetching plugin list: $pluginListUrl")
                    
                    try {
                        val request = Request.Builder().url(pluginListUrl).build()
                        httpClient.newCall(request).execute().use { response ->
                            if (response.isSuccessful) {
                                val pluginsBody = response.body.string()
                                if (!pluginsBody.isNullOrEmpty()) {
                                    // Try to parse as direct array first ([{...}, {...}])
                                    val pluginsArray = try {
                                        JSONArray(pluginsBody)
                                    } catch (e: Exception) {
                                        // Try as object with "plugins" key ({"plugins": [...]})
                                        try {
                                            JSONObject(pluginsBody).optJSONArray("plugins")
                                        } catch (e2: Exception) {
                                            null
                                        }
                                    }
                                    
                                    pluginsArray?.let { array ->
                                        Log.d(TAG, "Found ${array.length()} plugins in $pluginListUrl")
                                        for (j in 0 until array.length()) {
                                            try {
                                                val pluginObj = array.getJSONObject(j)
                                                val plugin = parsePluginInfo(pluginObj, pluginListUrl)
                                                if (plugin.url.isNotEmpty() && plugin.status == 1) { // Only active plugins
                                                    plugins.add(plugin)
                                                }
                                            } catch (e: Exception) {
                                                Log.w(TAG, "Error parsing plugin at index $j", e)
                                            }
                                        }
                                    } ?: Log.w(TAG, "No plugins array found in $pluginListUrl")
                                }
                            } else {
                                Log.w(TAG, "Failed to fetch plugin list: HTTP ${response.code}")
                            }
                        }
                    } catch (e: Exception) {
                        Log.e(TAG, "Error fetching plugin list $pluginListUrl", e)
                    }
                }
            }
            
            // Also check for direct plugins array (fallback)
            if (plugins.isEmpty()) {
                json.optJSONArray("plugins")?.let { pluginsArray ->
                    Log.d(TAG, "Found direct plugins array with ${pluginsArray.length()} entries")
                    for (i in 0 until pluginsArray.length()) {
                        val pluginObj = pluginsArray.getJSONObject(i)
                        val plugin = parsePluginInfo(pluginObj, repoUrl)
                        if (plugin.url.isNotEmpty()) {
                            plugins.add(plugin)
                        }
                    }
                }
            }
            
            Log.d(TAG, "Total plugins parsed: ${plugins.size}")
            
            RepositoryResult.Success(
                RepositoryManifest(
                    sourceUrl = repoUrl,
                    name = name,
                    description = description,
                    author = author,
                    version = version,
                    iconUrl = repoIconUrl,
                    plugins = plugins
                )
            )
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing repository manifest", e)
            RepositoryResult.Error("Parse error: ${e.message}")
        }
    }
    
    /**
     * Parse a single plugin info from JSON.
     * [baseUrl] is used to resolve relative iconUrl/url values (CS3 repos often use relative paths).
     */
    private fun parsePluginInfo(pluginObj: JSONObject, baseUrl: String? = null): PluginInfo {
        val rawIconUrl = firstNonBlank(
            pluginObj.optNullableString("iconUrl"),
            pluginObj.optNullableString("icon"),
            pluginObj.optNullableString("iconPath"),
            pluginObj.optNullableString("imageUrl"),
            pluginObj.optNullableString("logo")
        )
        val resolvedIconUrl = resolveUrl(rawIconUrl, baseUrl)

        val rawUrl = pluginObj.optString("url", "")
        val resolvedUrl = resolveUrl(rawUrl.takeIf { it.isNotBlank() }, baseUrl) ?: rawUrl

        return PluginInfo(
            name = pluginObj.optString("name", "Unknown"),
            internalName = pluginObj.optString("internalName", ""),
            description = pluginObj.optNullableString("description") ?: "",
            version = pluginObj.optInt("version", 1),
            url = resolvedUrl,
            sha256 = firstNonBlank(
                pluginObj.optNullableString("sha256"),
                pluginObj.optNullableString("fileHash")?.removePrefix("sha256-"),
                pluginObj.optNullableString("checksum"),
                pluginObj.optNullableString("hash")
            ),
            iconUrl = resolvedIconUrl,
            language = pluginObj.optString("language", "en"),
            tvTypes = parseTvTypes(pluginObj.optJSONArray("tvTypes")),
            status = pluginObj.optInt("status", 1)
        )
    }

    /**
     * Resolve a potentially relative URL against a base URL.
     * Returns the URL unchanged if already absolute, null if input is null.
     */
    private fun resolveUrl(url: String?, baseUrl: String?): String? {
        if (url.isNullOrBlank()) return null
        val resolved = if (url.startsWith("http://") || url.startsWith("https://")) {
            url
        } else if (baseUrl.isNullOrBlank()) {
            url
        } else {
            try {
                java.net.URL(java.net.URL(baseUrl), url).toString()
            } catch (e: Exception) {
                url
            }
        }
        // CS3 repos use %size% as a placeholder (e.g. Google favicons API)
        return resolved.replace("%size%", "128")
    }

    private fun JSONObject.optNullableString(name: String): String? {
        return if (has(name) && !isNull(name)) optString(name).takeIf { it.isNotBlank() } else null
    }

    /**
     * Parse TV types array
     */
    private fun parseTvTypes(array: org.json.JSONArray?): List<String> {
        if (array == null) return emptyList()
        
        return (0 until array.length()).mapNotNull { i ->
            try {
                array.getString(i)
            } catch (e: Exception) {
                null
            }
        }
    }

    private fun firstNonBlank(vararg values: String?): String? {
        return values.firstOrNull { !it.isNullOrBlank() }
    }

    /**
     * Validate a plugin URL by checking if the .cs3 file is accessible
     */
    suspend fun validatePluginUrl(pluginUrl: String): Boolean = withContext(Dispatchers.IO) {
        try {
            val request = Request.Builder()
                .url(pluginUrl)
                .head()
                .build()

            httpClient.newCall(request).execute().use { response ->
                response.isSuccessful && response.body.contentLength() < MAX_RESPONSE_SIZE
            }
        } catch (e: Exception) {
            Log.w(TAG, "Validation failed for $pluginUrl", e)
            false
        }
    }
}

// Result types for repository operations
sealed class RepositoryResult {
    data class Success(val manifest: RepositoryManifest) : RepositoryResult()
    data class Error(val message: String) : RepositoryResult()
}

// Data classes for repository structure
data class RepositoryManifest(
    val sourceUrl: String,
    val name: String,
    val description: String,
    val author: String,
    val version: Int,
    val iconUrl: String? = null,
    val plugins: List<PluginInfo>
)

data class PluginInfo(
    val name: String,
    val internalName: String,
    val description: String,
    val version: Int,
    val url: String,
    val sha256: String? = null,
    val iconUrl: String?,
    val language: String,
    val tvTypes: List<String>,
    val status: Int // 1 = active, 0 = deprecated
) {
    val isActive: Boolean get() = status == 1
    
    /**
     * Get display types for UI
     */
    fun getDisplayTypes(): String {
        return when {
            tvTypes.isEmpty() -> "All"
            tvTypes.size <= 3 -> tvTypes.joinToString(", ") { it.capitalize() }
            else -> "${tvTypes.take(3).joinToString(", ") { it.capitalize() }} +${tvTypes.size - 3}"
        }
    }
    
    private fun String.capitalize(): String {
        return replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
    }
}

// Local storage models
data class InstalledPlugin(
    val installId: String? = null,
    val internalName: String,
    val name: String,
    val description: String,
    val version: Int,
    val url: String,
    val filePath: String,
    val repositoryUrl: String?,
    val sha256: String? = null,
    val installedAt: Long = System.currentTimeMillis(),
    val iconUrl: String? = null
)

data class PluginRepositoryEntry(
    val url: String,
    val name: String,
    val description: String,
    val iconUrl: String? = null,
    val addedAt: Long = System.currentTimeMillis()
)
