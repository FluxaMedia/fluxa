package com.fluxa.app.data.repository

import android.content.Context
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.platform.AndroidPlatformFileStore
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AddonPersistentCache @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val cache = PlatformPersistentCache(AndroidPlatformFileStore(context.cacheDir))
    private val addonDescriptorType = object : TypeToken<AddonDescriptor>() {}.type
    private val addonListType = object : TypeToken<List<AddonDescriptor>>() {}.type

    suspend fun getManifest(key: String): AddonDescriptor? {
        return runCatching {
            val text = cache.read("manifest", key) ?: return null
            gson.fromJson<AddonDescriptor>(text, addonDescriptorType)
        }.getOrNull()
    }

    suspend fun putManifest(key: String, descriptor: AddonDescriptor) {
        runCatching {
            cache.write("manifest", key, gson.toJson(descriptor, addonDescriptorType))
        }
    }

    suspend fun getUserAddons(key: String): List<AddonDescriptor> {
        return runCatching {
            val text = cache.read("user_addons", key) ?: return emptyList()
            gson.fromJson<List<AddonDescriptor>>(text, addonListType).orEmpty()
        }.getOrDefault(emptyList())
    }

    suspend fun putUserAddons(key: String, addons: List<AddonDescriptor>) {
        if (addons.isEmpty()) return
        runCatching {
            cache.write("user_addons", key, gson.toJson(addons, addonListType))
        }
    }
}
