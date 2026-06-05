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
    private val client = okhttp3.OkHttpClient.Builder()
        .connectTimeout(3, java.util.concurrent.TimeUnit.SECONDS)
        .build()

    data class UpdateInfo(
        val versionCode: Int,
        val url: String,
        val releaseNotes: String?,
        val sha256: String? = null
    )

    suspend fun checkUpdate(serverUrl: String): UpdateInfo? = withContext(Dispatchers.IO) {
        val urlsToTry = mutableListOf<String>()

        if (BuildConfig.DEBUG) {
            val ips = listOf("localhost", "10.0.2.2")
            val ports = listOf(3000, 5050, 7000, 8000, 8080, 8888, 9999)
            for (ip in ips) {
                for (port in ports) {
                    urlsToTry.add("http://$ip:$port")
                }
            }
        }

        if (serverUrl.isNotEmpty()) {
            if (!BuildConfig.DEBUG && !serverUrl.startsWith("https://")) {
                Log.w("UpdateManager", "Ignoring non-HTTPS update URL in release build")
                return@withContext null
            }
            if (!urlsToTry.contains(serverUrl)) {
                urlsToTry.add(serverUrl)
            }
        }

        if (urlsToTry.isEmpty()) return@withContext null

        val fastClient = client.newBuilder()
            .connectTimeout(2, java.util.concurrent.TimeUnit.SECONDS) //  RELIABLE TIMEOUT
            .readTimeout(2, java.util.concurrent.TimeUnit.SECONDS)
            .build()

        for (baseUrl in urlsToTry) {
            try {
                Log.i("UpdateManager", "Checking: ${baseUrl}/update.json")
                val request = Request.Builder().url("${baseUrl.removeSuffix("/")}/update.json").build()
                fastClient.newCall(request).execute().use { response ->
                    if (!response.isSuccessful) {
                        Log.w("UpdateManager", "Failed response from $baseUrl: ${response.code}")
                        return@use
                    }
                    val responseBody = response.body
                    val body = responseBody.string()
                    val json = JSONObject(body)
                    val serverVersion = json.getInt("versionCode")
                    
                    Log.i("UpdateManager", "Server version: $serverVersion, Current version: ${BuildConfig.VERSION_CODE}")
                    
                    if (serverVersion > BuildConfig.VERSION_CODE) {
                        Log.i("UpdateManager", " Update FOUND! $serverVersion")
                        val downloadUrl = json.getString("url")
                        if (!BuildConfig.DEBUG && !downloadUrl.startsWith("https://")) {
                            Log.w("UpdateManager", "Ignoring update with non-HTTPS download URL")
                            return@use
                        }
                        return@withContext UpdateInfo(
                            versionCode = serverVersion,
                            url = downloadUrl,
                            releaseNotes = json.optString("notes"),
                            sha256 = json.optString("sha256").takeIf { it.isNotBlank() }
                        )
                    } else if (serverVersion == BuildConfig.VERSION_CODE) {
                        Log.i("UpdateManager", "Already up to date on $baseUrl")
                        return@withContext null
                    }
                }
            } catch (e: Exception) {
                Log.w("UpdateManager", "Skip $baseUrl: ${e.message}")
            }
        }
        Log.e("UpdateManager", "No update found after checking all URLs")
        null
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
            
            //  Intent for confirmation (Still needed on non-root)
            val intent = Intent(context, context.javaClass) // Placeholder, system handles it
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
