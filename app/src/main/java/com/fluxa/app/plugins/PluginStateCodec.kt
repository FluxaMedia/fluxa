package com.fluxa.app.plugins

import android.util.Log
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry
import org.json.JSONArray
import org.json.JSONObject
import java.security.MessageDigest

private const val TAG = "PluginStateCodec"

internal object PluginStateCodec {
    fun parseInstalledPlugins(json: String): List<InstalledPlugin> {
        return try {
            val array = JSONArray(json)
            (0 until array.length()).map { i ->
                val obj = array.getJSONObject(i)
                InstalledPlugin(
                    installId = obj.optString("installId").takeIf { it.isNotBlank() },
                    internalName = obj.getString("internalName"),
                    name = obj.getString("name"),
                    description = obj.optString("description", ""),
                    version = obj.optInt("version", 1),
                    url = obj.getString("url"),
                    filePath = obj.getString("filePath"),
                    repositoryUrl = obj.optString("repositoryUrl").takeIf { it.isNotBlank() },
                    sha256 = obj.optString("sha256").takeIf { it.isNotBlank() },
                    installedAt = obj.optLong("installedAt", System.currentTimeMillis()),
                    iconUrl = obj.optString("iconUrl").takeIf { it.isNotBlank() }
                )
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing installed plugins", e)
            emptyList()
        }
    }

    fun installedPluginsToJson(plugins: List<InstalledPlugin>): String {
        val array = JSONArray()
        plugins.forEach { plugin ->
            val obj = JSONObject().apply {
                plugin.installId?.let { put("installId", it) }
                put("internalName", plugin.internalName)
                put("name", plugin.name)
                put("description", plugin.description)
                put("version", plugin.version)
                put("url", plugin.url)
                put("filePath", plugin.filePath)
                put("repositoryUrl", plugin.repositoryUrl)
                plugin.sha256?.let { put("sha256", it) }
                put("installedAt", plugin.installedAt)
                plugin.iconUrl?.let { put("iconUrl", it) }
            }
            array.put(obj)
        }
        return array.toString()
    }

    fun parseRepositories(json: String): List<PluginRepositoryEntry> {
        return try {
            val array = JSONArray(json)
            (0 until array.length()).map { i ->
                val obj = array.getJSONObject(i)
                PluginRepositoryEntry(
                    url = obj.getString("url"),
                    name = obj.optString("name", ""),
                    description = obj.optString("description", ""),
                    iconUrl = obj.optString("iconUrl").takeIf { it.isNotBlank() },
                    addedAt = obj.optLong("addedAt", System.currentTimeMillis())
                )
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing repositories", e)
            emptyList()
        }
    }

    fun repositoriesToJson(repos: List<PluginRepositoryEntry>): String {
        val array = JSONArray()
        repos.forEach { repo ->
            val obj = JSONObject().apply {
                put("url", repo.url)
                put("name", repo.name)
                put("description", repo.description)
                repo.iconUrl?.let { put("iconUrl", it) }
                put("addedAt", repo.addedAt)
            }
            array.put(obj)
        }
        return array.toString()
    }

    fun sha256(input: String): String {
        val bytes = MessageDigest.getInstance("SHA-256").digest(input.toByteArray())
        return bytes.joinToString("") { "%02x".format(it) }
    }
}
