package com.fluxa.app.data.repository

import android.content.Context
import com.fluxa.app.data.remote.AddonDescriptor
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import dagger.hilt.android.qualifiers.ApplicationContext
import java.io.File
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AddonPersistentCache @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val cacheDir = File(context.cacheDir, "addon_cache").also { it.mkdirs() }
    private val addonDescriptorType = object : TypeToken<AddonDescriptor>() {}.type
    private val addonListType = object : TypeToken<List<AddonDescriptor>>() {}.type

    private fun fileFor(prefix: String, key: String): File {
        val safe = key.replace(Regex("[^A-Za-z0-9_\\-]"), "_").take(200)
        return File(cacheDir, "${prefix}_$safe.json")
    }

    fun getManifest(key: String): AddonDescriptor? {
        return runCatching {
            val text = fileFor("manifest", key).takeIf { it.exists() }?.readText() ?: return null
            gson.fromJson<AddonDescriptor>(text, addonDescriptorType)
        }.getOrNull()
    }

    fun putManifest(key: String, descriptor: AddonDescriptor) {
        runCatching {
            fileFor("manifest", key).writeText(gson.toJson(descriptor, addonDescriptorType))
        }
    }

    fun getUserAddons(key: String): List<AddonDescriptor> {
        return runCatching {
            val text = fileFor("user_addons", key).takeIf { it.exists() }?.readText() ?: return emptyList()
            gson.fromJson<List<AddonDescriptor>>(text, addonListType).orEmpty()
        }.getOrDefault(emptyList())
    }

    fun putUserAddons(key: String, addons: List<AddonDescriptor>) {
        if (addons.isEmpty()) return
        runCatching {
            fileFor("user_addons", key).writeText(gson.toJson(addons, addonListType))
        }
    }
}
