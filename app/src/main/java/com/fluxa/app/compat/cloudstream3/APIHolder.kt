package com.fluxa.app.compat.cloudstream3

import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.utils.ExtractorApi

/**
 * Compat wrapper around real CloudStream library APIHolder.
 * Provides additional utility methods not present in the real library.
 */
object APIHolder {
    /** Delegate to real library's APIHolder.allProviders */
    val allProviders: MutableList<MainAPI>
        get() = com.lagradost.cloudstream3.APIHolder.allProviders
    
    /**
     * Delegate to real library's addPluginMapping
     */
    fun addPluginMapping(api: MainAPI) {
        com.lagradost.cloudstream3.APIHolder.addPluginMapping(api)
    }
    
    /**
     * Remove a specific plugin's MainAPI from the registry
     */
    fun removePluginMapping(api: MainAPI) {
        com.lagradost.cloudstream3.APIHolder.removePluginMapping(api)
    }
    
    /**
     * Remove all providers from a specific plugin by filename
     */
    fun removePluginMappings(pluginFilename: String?) {
        if (pluginFilename == null) return
        synchronized(allProviders) {
            allProviders.removeAll { it.sourcePlugin == pluginFilename }
        }
    }
    
    /**
     * Get all registered providers
     */
    fun getProviders(): List<MainAPI> {
        return synchronized(allProviders) {
            allProviders.toList()
        }
    }
    
    /**
     * Clear all providers (used for testing or reset)
     */
    fun clearProviders() {
        synchronized(allProviders) {
            allProviders.clear()
        }
    }
    
    /**
     * Initialize global extractor registry.
     * This should be called before loading any plugins.
     */
    fun installGlobalExtractors() {
        // The real CloudStream library initializes extractorApis automatically
        // This is a placeholder for any additional initialization needed
        synchronized(com.lagradost.cloudstream3.utils.extractorApis) {
            // Ensure the global list is initialized
            @Suppress("UNUSED_VARIABLE")
            val extractors = com.lagradost.cloudstream3.utils.extractorApis.toList()
        }
    }
    
    /**
     * Register all extractors from a plugin to the global list.
     * This is called after plugin.load() to ensure extractors are available.
     */
    fun registerAllExtractors(extractors: List<ExtractorApi>) {
        synchronized(com.lagradost.cloudstream3.utils.extractorApis) {
            extractors.forEach { extractor ->
                if (!com.lagradost.cloudstream3.utils.extractorApis.contains(extractor)) {
                    com.lagradost.cloudstream3.utils.extractorApis.add(extractor)
                }
            }
        }
    }
}
