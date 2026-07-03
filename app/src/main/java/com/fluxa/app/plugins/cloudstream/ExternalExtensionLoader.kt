package com.fluxa.app.plugins.cloudstream

import android.content.Context
import android.content.ContextWrapper
import android.content.res.AssetManager
import android.content.res.Resources
import android.os.Build
import android.util.Log
import com.fluxa.app.data.repository.HttpRequestSecurity
import com.lagradost.cloudstream3.AcraApplication
import com.fluxa.app.compat.cloudstream3.APIHolder
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.plugins.BasePlugin
import com.lagradost.cloudstream3.plugins.Plugin
import com.lagradost.cloudstream3.utils.extractorApis
import dalvik.system.PathClassLoader
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import java.util.zip.ZipFile

private const val TAG = "ExternalExtensionLoader"
private const val MAX_DEX_SIZE = 10 * 1024 * 1024L // 10MB max per .cs3 file
private const val CLOUDSTREAM_PACKAGE_NAME = "com.lagradost.cloudstream3"

/**
 * Manages downloading, loading, and caching of DEX-based external extensions (.cs3 files).
 */
class ExternalExtensionLoader(
    private val context: Context
) {
    private val httpClient = OkHttpClient.Builder()
        .addInterceptor { chain ->
            chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
        }
        .connectTimeout(60, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .followRedirects(true)
        .build()

    /** Cache of loaded MainAPI instances by scraper ID (maps to multiple APIs per scraper) */
    private val apiCache = ConcurrentHashMap<String, List<MainAPI>>()

    /** Cache of loaded class loaders by scraper ID */
    private val classLoaderCache = ConcurrentHashMap<String, ClassLoader>()

    private val extensionsDir: File by lazy {
        File(context.filesDir, "cs_extensions").apply { mkdirs() }
    }
    
    /** Sanitize scraper ID for use as a filename. */
    private fun safeFileName(scraperId: String): String =
        scraperId.replace(':', '_').replace('/', '_')

    /**
     * Download a .cs3 DEX file for the given scraper.
     */
    suspend fun downloadExtension(scraperId: String, downloadUrl: String): File? = withContext(Dispatchers.IO) {
        try {
            val targetFile = File(extensionsDir, "${safeFileName(scraperId)}.cs3")

            if (targetFile.exists()) {
                targetFile.setWritable(true)
                targetFile.delete()
            }

            val request = Request.Builder()
                .url(downloadUrl)
                .header("User-Agent", com.fluxa.app.BuildConfig.APPLICATION_ID)
                .build()

            httpClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) {
                    Log.e(TAG, "Failed to download $scraperId: ${response.code}")
                    return@withContext null
                }

                val body = response.body.bytes()
                if (body.size > MAX_DEX_SIZE) {
                    Log.e(TAG, "Extension $scraperId exceeds size limit")
                    return@withContext null
                }

                targetFile.writeBytes(body)
                
                // Security fix for Android 14+
                targetFile.setReadOnly()
                
                Log.d(TAG, "Downloaded extension $scraperId to ${targetFile.absolutePath}")
                targetFile
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error downloading extension $scraperId", e)
            null
        }
    }

    /**
     * Load MainAPIs from a .cs3 file.
     */
    suspend fun loadApisFromFile(scraperId: String, file: File): List<MainAPI> = withContext(Dispatchers.IO) {
        try {
            if (apiCache.containsKey(scraperId)) return@withContext apiCache[scraperId] ?: emptyList()

            // Android 14+ requirement
            ensureDexReadOnly(file)
            val classLoader = createClassLoader(file)
            classLoaderCache[scraperId] = classLoader

            val plugin = findAndLoadPlugin(classLoader, file) ?: run {
                Log.e(TAG, "Could not find plugin class for $scraperId")
                return@withContext emptyList()
            }

            // Handle resources if needed
            plugin.filename = file.absolutePath
            val manifest = readManifestFromZip(file)
            if (manifest?.optBoolean("requiresResources", false) == true) {
                loadPluginResources(plugin, file)
            }

            // Load plugin
            val providersBefore = synchronized(APIHolder.allProviders) { APIHolder.allProviders.toList() }
            val pluginContext = CloudstreamPluginContext(AcraApplication.getActivity() ?: context)
            try {
                (plugin as? Plugin)?.load(pluginContext) ?: plugin.load()
            } catch (t: Throwable) {
                Log.e(TAG, "Plugin load() failed for $scraperId", t)
                unloadExtension(scraperId)
                return@withContext emptyList()
            }

            val apis = synchronized(APIHolder.allProviders) {
                APIHolder.allProviders.toList()
            }.filterNot { it in providersBefore }.toMutableList()
            
            if (apis.isNotEmpty()) {
                apiCache[scraperId] = apis
                Log.d(TAG, "Successfully loaded ${apis.size} APIs from $scraperId")
            }

            apis
        } catch (t: Throwable) {
            Log.e(TAG, "Error loading APIs from $scraperId", t)
            emptyList()
        }
    }

    private fun ensureDexReadOnly(file: File) {
        if (file.canWrite()) file.setReadOnly()
    }

    private fun createClassLoader(file: File): ClassLoader =
        PathClassLoader(file.absolutePath, context.classLoader)

    private fun readManifestFromZip(file: File): JSONObject? {
        return try {
            ZipFile(file).use { zip ->
                val entry = zip.getEntry("manifest.json") ?: return null
                val content = zip.getInputStream(entry).readBytes().toString(Charsets.UTF_8)
                JSONObject(content)
            }
        } catch (e: Exception) { null }
    }

    private fun findAndLoadPlugin(classLoader: ClassLoader, file: File): BasePlugin? {
        val manifest = readManifestFromZip(file)
        val className = manifest?.optString("pluginClassName")
        if (className.isNullOrBlank()) return null
        return try {
            val clazz = classLoader.loadClass(className)
            clazz.getDeclaredConstructor().newInstance() as? BasePlugin
        } catch (t: Throwable) {
            Log.w(TAG, "Failed to load plugin class $className", t)
            null
        }
    }

    @Suppress("DEPRECATION")
    private fun loadPluginResources(plugin: BasePlugin, file: File) {
        try {
            val assets = AssetManager::class.java.getDeclaredConstructor().newInstance()
            val addAssetPath = AssetManager::class.java.getMethod("addAssetPath", String::class.java)
            addAssetPath.invoke(assets, file.absolutePath)
            (plugin as? Plugin)?.resources = Resources(assets, context.resources.displayMetrics, context.resources.configuration)
            Log.d(TAG, "Loaded resources for ${plugin::class.java.simpleName}")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to load plugin resources: ${e.message}")
        }
    }

    fun unloadExtension(scraperId: String) {
        val apis = apiCache.remove(scraperId)
        apis?.forEach {
            APIHolder.removePluginMapping(it)
        }
        apis?.firstOrNull()?.sourcePlugin?.let { sourcePlugin ->
            APIHolder.removePluginMappings(sourcePlugin)
            extractorApis.removeAll { it.sourcePlugin == sourcePlugin }
        }
        classLoaderCache.remove(scraperId)
    }

    fun clearCaches() {
        apiCache.clear()
        classLoaderCache.clear()
    }

    fun deleteExtensionFile(scraperId: String): Boolean {
        val file = File(extensionsDir, "${safeFileName(scraperId)}.cs3")
        return file.delete()
    }

    private class CloudstreamPluginContext(base: Context) : ContextWrapper(base) {
        override fun getPackageName(): String = CLOUDSTREAM_PACKAGE_NAME
        override fun getApplicationContext(): Context = this
    }
}
