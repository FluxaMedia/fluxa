package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.util.Log
import androidx.core.content.FileProvider
import com.fluxa.app.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.security.MessageDigest

object UpdateManager {
    private const val RELEASES_URL = "https://api.github.com/repos/KhooLy/Fluxa/releases/latest"

    private val client = okhttp3.OkHttpClient.Builder()
        .connectTimeout(5, java.util.concurrent.TimeUnit.SECONDS)
        .readTimeout(5, java.util.concurrent.TimeUnit.SECONDS)
        .build()

    data class UpdateInfo(
        val versionName: String,
        val url: String,
        val releaseNotes: String?,
        val sha256: String? = null
    )

    suspend fun checkUpdate(): UpdateInfo? = withContext(Dispatchers.IO) {
        try {
            val request = Request.Builder()
                .url(RELEASES_URL)
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "Fluxa-App")
                .build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) {
                    Log.w("UpdateManager", "GitHub releases request failed: ${response.code}")
                    return@withContext null
                }
                val json = JSONObject(response.body.string())
                val tagName = json.getString("tag_name").removePrefix("v")

                Log.i("UpdateManager", "Latest release: $tagName, current: ${BuildConfig.VERSION_NAME}")

                if (!isNewerVersion(tagName, BuildConfig.VERSION_NAME)) {
                    Log.i("UpdateManager", "Already up to date")
                    return@withContext null
                }

                val assets = json.getJSONArray("assets")
                val apkAssets = (0 until assets.length())
                    .map { assets.getJSONObject(it) }
                    .filter { it.getString("name").endsWith(".apk") }

                val apkUrl = findMatchingApk(apkAssets)
                if (apkUrl == null) {
                    Log.w("UpdateManager", "No matching APK asset found for ABIs ${Build.SUPPORTED_ABIS.joinToString()}")
                    return@withContext null
                }

                UpdateInfo(
                    versionName = tagName,
                    url = apkUrl,
                    releaseNotes = json.optString("body").takeIf { it.isNotBlank() }
                )
            }
        } catch (e: Exception) {
            Log.w("UpdateManager", "Update check failed: ${e.message}")
            null
        }
    }

    private fun findMatchingApk(apkAssets: List<JSONObject>): String? {
        val flavorMatches = apkAssets.filter {
            it.getString("name").contains(BuildConfig.DEVICE_FLAVOR, ignoreCase = true)
        }
        val candidates = flavorMatches.ifEmpty { apkAssets }

        for (abi in Build.SUPPORTED_ABIS) {
            val match = candidates.find { it.getString("name").contains(abi, ignoreCase = true) }
            if (match != null) return match.getString("browser_download_url")
        }

        return candidates.find { it.getString("name").contains("universal", ignoreCase = true) }
            ?.getString("browser_download_url")
    }

    private fun isNewerVersion(remote: String, current: String): Boolean {
        val remoteParts = remote.split(".").map { it.toIntOrNull() ?: 0 }
        val currentParts = current.split(".").map { it.toIntOrNull() ?: 0 }
        val size = maxOf(remoteParts.size, currentParts.size)
        for (i in 0 until size) {
            val r = remoteParts.getOrElse(i) { 0 }
            val c = currentParts.getOrElse(i) { 0 }
            if (r != c) return r > c
        }
        return false
    }

    suspend fun downloadAndInstall(
        context: Context,
        updateUrl: String,
        expectedSha256: String? = null,
        onProgress: (Float) -> Unit
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            if (!BuildConfig.DEBUG && !updateUrl.startsWith("https://")) {
                Log.e("UpdateManager", "Refusing non-HTTPS update download in release build")
                return@withContext false
            }
            Log.i("UpdateManager", "Starting download from $updateUrl")
            val request = Request.Builder().url(updateUrl).build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) {
                    Log.e("UpdateManager", "Server returned error: ${response.code}")
                    return@withContext false
                }
                
                val cacheDir = context.externalCacheDir ?: context.cacheDir
                val apkFile = File(cacheDir, "update.apk")
                val body = response.body
                val totalSize = body.contentLength()
                
                Log.i("UpdateManager", "APK Size: $totalSize bytes")
                
                body.byteStream().use { input ->
                    FileOutputStream(apkFile).use { output ->
                        val buffer = ByteArray(16 * 1024)
                        var bytesRead: Int
                        var totalRead = 0L
                        while (input.read(buffer).also { bytesRead = it } != -1) {
                            output.write(buffer, 0, bytesRead)
                            totalRead += bytesRead
                            if (totalSize > 0) {
                                onProgress(totalRead.toFloat() / totalSize)
                            }
                        }
                    }
                }

                if (!expectedSha256.isNullOrBlank()) {
                    val actualSha256 = apkFile.sha256()
                    if (!actualSha256.equals(expectedSha256, ignoreCase = true)) {
                        Log.e("UpdateManager", "APK checksum mismatch. expected=$expectedSha256 actual=$actualSha256")
                        apkFile.delete()
                        return@withContext false
                    }
                }
                
                Log.i("UpdateManager", "Download complete, starting installation")
                installApk(context, apkFile)
                return@withContext true
            }
        } catch (e: Exception) {
            Log.e("UpdateManager", "Download failed: ${e.message}", e)
            return@withContext false
        }
    }

    private fun installApk(context: Context, file: File) {
        try {
            val packageInstaller = context.packageManager.packageInstaller
            val params = android.content.pm.PackageInstaller.SessionParams(
                android.content.pm.PackageInstaller.SessionParams.MODE_FULL_INSTALL
            )
            val sessionId = packageInstaller.createSession(params)
            val session = packageInstaller.openSession(sessionId)
            
            file.inputStream().use { input ->
                session.openWrite("update", 0, file.length()).use { output ->
                    input.copyTo(output)
                    session.fsync(output)
                }
            }
            
            val intent = Intent(context, context.javaClass)
            val pendingIntent = android.app.PendingIntent.getBroadcast(
                context, sessionId, Intent("com.fluxa.app.INSTALL_COMPLETE"),
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) android.app.PendingIntent.FLAG_MUTABLE else 0
            )
            
            session.commit(pendingIntent.intentSender)
            session.close()
        } catch (e: Exception) {
            Log.e("UpdateManager", "PackageInstaller failed, using legacy", e)
            legacyInstall(context, file)
        }
    }

    private fun legacyInstall(context: Context, file: File) {
        val uri = FileProvider.getUriForFile(context, "${context.packageName}.fileprovider", file)
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(uri, "application/vnd.android.package-archive")
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        context.startActivity(intent)
    }

    private fun File.sha256(): String {
        val digest = MessageDigest.getInstance("SHA-256")
        inputStream().use { input ->
            val buffer = ByteArray(16 * 1024)
            var read: Int
            while (input.read(buffer).also { read = it } != -1) {
                digest.update(buffer, 0, read)
            }
        }
        return digest.digest().joinToString("") { byte -> "%02x".format(byte) }
    }
}
