package com.fluxa.app.plugins.cloudstream

import android.content.Context
import android.content.res.AssetManager
import android.content.res.Resources
import android.os.Build
import android.util.Log
import com.fluxa.app.data.repository.HttpRequestSecurity
import com.lagradost.cloudstream3.AcraApplication
import com.fluxa.app.compat.cloudstream3.APIHolder
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.utils.ExtractorApi
import com.lagradost.cloudstream3.plugins.BasePlugin
import com.lagradost.cloudstream3.plugins.CloudstreamPlugin
import com.lagradost.cloudstream3.plugins.Plugin
import com.lagradost.cloudstream3.utils.extractorApis
import dalvik.system.InMemoryDexClassLoader
import dalvik.system.PathClassLoader
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.nio.ByteBuffer
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import java.util.zip.ZipFile

private const val TAG = "ExternalExtensionLoader"
private const val MAX_DEX_SIZE = 10 * 1024 * 1024L // 10MB max per .cs3 file

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
    
    private val codeCacheDir: File by lazy {
        File(context.codeCacheDir, "cs_dex_cache").apply { mkdirs() }
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
            val loadableFile = ensureLoadablePluginContainer(scraperId, file)
            val classLoader = createClassLoader(scraperId, file, loadableFile)
            classLoaderCache[scraperId] = classLoader

            val plugin = findAndLoadPlugin(classLoader, loadableFile) ?: run {
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
            val activity = AcraApplication.getActivity()
            try {
                if (activity != null) {
                    plugin.load(activity)
                } else {
                    plugin.load(context)
                }
            } catch (t: Throwable) {
                Log.w(TAG, "Plugin load() failed for $scraperId", t)
            }

            val apis = synchronized(APIHolder.allProviders) {
                APIHolder.allProviders.toList()
            }.filterNot { it in providersBefore }.toMutableList()
            
            // Fallback: Scan DEX if no APIs registered
            if (apis.isEmpty()) {
                val dexApis = scanDexForApis(classLoader, loadableFile)
                dexApis.forEach { it.sourcePlugin = file.absolutePath }
                apis.addAll(dexApis)
            }

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

    private fun ensureLoadablePluginContainer(scraperId: String, file: File): File {
        if (file.extension.equals("apk", ignoreCase = true) || file.extension.equals("jar", ignoreCase = true)) {
            return file
        }
        val apkMirror = File(codeCacheDir, "${safeFileName(scraperId)}.apk")
        if (!apkMirror.exists() || apkMirror.length() != file.length() || apkMirror.lastModified() < file.lastModified()) {
            file.copyTo(apkMirror, overwrite = true)
            apkMirror.setReadOnly()
            Log.d(TAG, "Prepared APK mirror for plugin $scraperId at ${apkMirror.absolutePath}")
        }
        return apkMirror
    }

    private fun createClassLoader(scraperId: String, originalFile: File, loadableFile: File): ClassLoader {
        val pathLoader = PathClassLoader(loadableFile.absolutePath, context.classLoader)
        val pluginClassName = readManifestFromZip(originalFile)?.optString("pluginClassName")
        if (pluginClassName.isNullOrBlank()) {
            return pathLoader
        }
        return try {
            pathLoader.loadClass(pluginClassName)
            pathLoader
        } catch (t: Throwable) {
            Log.w(TAG, "PathClassLoader validation failed for $scraperId ($pluginClassName), falling back to in-memory DEX", t)
            createInMemoryDexClassLoader(originalFile)
        }
    }

    private fun createInMemoryDexClassLoader(file: File): ClassLoader {
        ZipFile(file).use { zip ->
            val dexEntry = zip.getEntry("classes.dex") ?: error("classes.dex missing in ${file.name}")
            val dexBytes = zip.getInputStream(dexEntry).readBytes()
            return InMemoryDexClassLoader(ByteBuffer.wrap(dexBytes), context.classLoader)
        }
    }

    private fun readManifestFromZip(file: File): JSONObject? {
        return try {
            ZipFile(file).use { zip ->
                val entry = zip.getEntry("manifest.json") ?: return null
                val content = zip.getInputStream(entry).readBytes().toString(Charsets.UTF_8)
                JSONObject(content)
            }
        } catch (e: Exception) { null }
    }

    private fun findAndLoadPlugin(classLoader: ClassLoader, file: File): Plugin? {
        val manifest = readManifestFromZip(file)
        val className = manifest?.optString("pluginClassName")
        
        if (!className.isNullOrBlank()) {
            try {
                val clazz = classLoader.loadClass(className)
                val instance = clazz.getDeclaredConstructor().newInstance()
                if (instance is Plugin) return instance
                if (looksLikePlugin(instance)) return ReflectivePluginWrapper(instance)
            } catch (t: Throwable) {
                Log.w(TAG, "Failed to load plugin class $className", t)
            }
        }
        
        // Fallback: Scan for annotation or type
        return scanForPluginClass(classLoader, file)
    }

    @Suppress("DEPRECATION")
    private fun scanForPluginClass(classLoader: ClassLoader, file: File): Plugin? {
        try {
            val dex = dalvik.system.DexFile(file)
            val entries = dex.entries()
            while (entries.hasMoreElements()) {
                val name = entries.nextElement()
                try {
                    val clazz = classLoader.loadClass(name)
                    if (clazz.isAnnotationPresent(CloudstreamPlugin::class.java)) {
                        val instance = clazz.getDeclaredConstructor().newInstance()
                        if (instance is Plugin) return instance
                        if (looksLikePlugin(instance)) return ReflectivePluginWrapper(instance)
                    }
                } catch (_: Throwable) {}
            }
        } catch (t: Throwable) {
            Log.w(TAG, "DEX scan failed", t)
        }
        return null
    }

    @Suppress("DEPRECATION")
    private fun scanDexForApis(classLoader: ClassLoader, file: File): List<MainAPI> {
        val apis = mutableListOf<MainAPI>()
        try {
            val dex = dalvik.system.DexFile(file)
            val entries = dex.entries()
            while (entries.hasMoreElements()) {
                val name = entries.nextElement()
                if (name.contains("$") || name.contains("Plugin")) continue
                try {
                    val clazz = classLoader.loadClass(name)
                    if (MainAPI::class.java.isAssignableFrom(clazz) && 
                        !java.lang.reflect.Modifier.isAbstract(clazz.modifiers)) {
                        val instance = clazz.getDeclaredConstructor().newInstance() as MainAPI
                        apis.add(instance)
                    }
                } catch (_: Throwable) {}
            }
        } catch (_: Throwable) {}
        return apis
    }

    @Suppress("DEPRECATION")
    private fun loadPluginResources(plugin: Plugin, file: File) {
        try {
            val assets = AssetManager::class.java.getDeclaredConstructor().newInstance()
            val addAssetPath = AssetManager::class.java.getMethod("addAssetPath", String::class.java)
            addAssetPath.invoke(assets, file.absolutePath)
            plugin.resources = Resources(assets, context.resources.displayMetrics, context.resources.configuration)
            Log.d(TAG, "Loaded resources for ${plugin.name}")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to load plugin resources: ${e.message}")
        }
    }

    fun unloadExtension(scraperId: String) {
        val apis = apiCache.remove(scraperId)
        apis?.forEach {
            APIHolder.removePluginMapping(it)
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
}

/**
 * Checks whether an instance looks like a CloudStream plugin.
 */
private fun looksLikePlugin(instance: Any): Boolean {
    if (instance is BasePlugin) return true
    val clazz = instance.javaClass
    return try {
        clazz.getMethod("load", Context::class.java) != null || 
        clazz.getMethod("load") != null
    } catch (_: Exception) {
        try {
            clazz.getMethod("getRegisteredMainAPIs") != null
        } catch (_: Exception) { false }
    }
}

/**
 * Wraps a plugin instance loaded with a non-standard base class.
 */
private class ReflectivePluginWrapper(private val foreignInstance: Any) : Plugin() {
    override fun load(context: Context) {
        val providersBefore = synchronized(APIHolder.allProviders) { APIHolder.allProviders.toList() }
        val extractorsBefore = extractorApis.toList()
        val clazz = foreignInstance.javaClass

        // Try load(Context)
        try {
            clazz.getMethod("load", Context::class.java).invoke(foreignInstance, context)
        } catch (_: Throwable) {
            try {
                clazz.getMethod("load").invoke(foreignInstance)
            } catch (_: Throwable) {}
        }

        // Collect registered APIs
        try {
            val apis = clazz.getMethod("getRegisteredMainAPIs").invoke(foreignInstance) as? List<*>
            apis?.filterIsInstance<MainAPI>()?.forEach { registerMainAPI(it) }
        } catch (_: Throwable) {
            val newProviders = synchronized(APIHolder.allProviders) { APIHolder.allProviders.toList() } - providersBefore.toSet()
            newProviders.forEach { registerMainAPI(it) }
        }

        // Collect registered Extractors
        try {
            val extractors = clazz.getMethod("getRegisteredExtractorAPIs").invoke(foreignInstance) as? List<*>
            extractors?.filterIsInstance<ExtractorApi>()?.forEach { registerExtractorAPI(it) }
        } catch (_: Throwable) {
            val newExtractors = extractorApis.toList() - extractorsBefore.toSet()
            newExtractors.forEach { registerExtractorAPI(it) }
        }
    }
}
