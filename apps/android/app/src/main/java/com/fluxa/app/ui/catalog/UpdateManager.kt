package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
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
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.security.MessageDigest

object UpdateManager {
    private const val RELEASES_URL = "https://api.github.com/repos/FluxaMedia/fluxa/releases?per_page=100"
    private const val CONTRIBUTORS_URL = "https://api.github.com/repos/FluxaMedia/fluxa/contributors?per_page=100"
    private const val RECENT_COMMITS_URL = "https://api.github.com/repos/FluxaMedia/fluxa/commits?per_page=100"
    private const val SUPPORTERS_URL = "https://raw.githubusercontent.com/FluxaMedia/fluxa/master/shared/supporters.json"

    private val client = okhttp3.OkHttpClient.Builder()
        .connectTimeout(5, java.util.concurrent.TimeUnit.SECONDS)
        .readTimeout(5, java.util.concurrent.TimeUnit.SECONDS)
        .build()

    data class UpdateInfo(
        val versionName: String,
        val url: String,
        val releaseNotes: String?,
        val sha256: String? = null,
        val publishedAt: String? = null,
        val contributors: List<ContributorInfo> = emptyList(),
    )

    data class ContributorInfo(
        val login: String,
        val avatarUrl: String? = null,
        val profileUrl: String? = null,
        val contributions: Int = 0,
        val latestContributionAt: String? = null,
    )

    data class SupporterInfo(
        val login: String,
        val displayName: String? = null,
        val avatarUrl: String? = null,
        val profileUrl: String? = null,
        val supportCount: Int = 1,
        val supportedAt: String? = null,
    )

    data class CommunityInfo(
        val contributors: List<ContributorInfo> = emptyList(),
        val supporters: List<SupporterInfo> = emptyList(),
        val loadFailed: Boolean = false,
    )

    suspend fun fetchLatestRelease(): UpdateInfo? = fetchReleaseHistory().firstOrNull()

    suspend fun fetchReleaseHistory(): List<UpdateInfo> = withContext(Dispatchers.IO) {
        try {
            val request = Request.Builder()
                .url(RELEASES_URL)
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "Fluxa-App")
                .build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) {
                    Log.w("UpdateManager", "GitHub releases request failed: ${response.code}")
                    return@withContext emptyList()
                }
                val releases = JSONArray(response.body.string())
                buildList {
                    for (index in 0 until releases.length()) {
                        val release = releases.getJSONObject(index)
                        val assets = release.optJSONArray("assets") ?: continue
                        val apkAssets = (0 until assets.length())
                            .map { assets.getJSONObject(it) }
                            .filter { it.optString("name").endsWith(".apk", ignoreCase = true) }
                        val apkUrl = findMatchingApk(apkAssets) ?: continue
                        val tagName = release.optString("tag_name").removePrefix("v")
                        if (tagName.isBlank()) continue

                        Log.i("UpdateManager", "Compatible ${BuildConfig.DEVICE_FLAVOR} release: $tagName, current: ${BuildConfig.VERSION_NAME}")
                        add(UpdateInfo(
                            versionName = tagName,
                            url = apkUrl,
                            releaseNotes = release.optString("body").takeIf { it.isNotBlank() },
                            publishedAt = release.optString("published_at").takeIf { it.isNotBlank() },
                            contributors = parseReleaseContributors(release.optString("body")),
                        ))
                    }
                }
                    .also { releasesForDevice ->
                        if (releasesForDevice.isEmpty()) {
                            Log.i("UpdateManager", "No compatible ${BuildConfig.DEVICE_FLAVOR} release found for ABIs ${Build.SUPPORTED_ABIS.joinToString()}")
                        }
                    }
            }
        } catch (e: Exception) {
            Log.w("UpdateManager", "Update check failed: ${e.message}")
            emptyList()
        }
    }

    suspend fun fetchCommunityInfo(): CommunityInfo = withContext(Dispatchers.IO) {
        try {
            val contributorsResponse = getJsonArray(CONTRIBUTORS_URL)
            val recentCommitsResponse = getJsonArray(RECENT_COMMITS_URL)
            val latestByLogin = mutableMapOf<String, String>()
            for (index in 0 until recentCommitsResponse.length()) {
                val commit = recentCommitsResponse.optJSONObject(index) ?: continue
                val login = commit.optJSONObject("author")?.optString("login").orEmpty()
                val date = commit.optJSONObject("commit")?.optJSONObject("author")?.optString("date").orEmpty()
                if (login.isNotBlank() && date.isNotBlank() && date > latestByLogin[login].orEmpty()) {
                    latestByLogin[login] = date
                }
            }
            val contributors = buildList {
                for (index in 0 until contributorsResponse.length()) {
                    val contributor = contributorsResponse.optJSONObject(index) ?: continue
                    val login = contributor.optString("login").takeIf { it.isNotBlank() } ?: continue
                    add(
                        ContributorInfo(
                            login = login,
                            avatarUrl = contributor.optString("avatar_url").takeIf { it.isNotBlank() },
                            profileUrl = contributor.optString("html_url").takeIf { it.isNotBlank() },
                            contributions = contributor.optInt("contributions"),
                            latestContributionAt = latestByLogin[login],
                        )
                    )
                }
            }
            CommunityInfo(
                contributors = contributors,
                supporters = fetchSupporters(),
            )
        } catch (e: Exception) {
            Log.w("UpdateManager", "Community data request failed: ${e.message}")
            CommunityInfo(loadFailed = true)
        }
    }

    private fun fetchSupporters(): List<SupporterInfo> {
        return runCatching {
            val source = getJsonObject(SUPPORTERS_URL)
            val supporters = source.optJSONArray("supporters") ?: return@runCatching emptyList()
            buildList {
                for (index in 0 until supporters.length()) {
                    val supporter = supporters.optJSONObject(index) ?: continue
                    val login = supporter.optString("login").takeIf { it.isNotBlank() } ?: continue
                    add(
                        SupporterInfo(
                            login = login,
                            displayName = supporter.optString("displayName").takeIf { it.isNotBlank() },
                            avatarUrl = supporter.optString("avatarUrl").takeIf { it.isNotBlank() },
                            profileUrl = supporter.optString("profileUrl").takeIf { it.isNotBlank() },
                            supportCount = supporter.optInt("supportCount", 1).coerceAtLeast(1),
                            supportedAt = supporter.optString("supportedAt").takeIf { it.isNotBlank() },
                        )
                    )
                }
            }
        }.getOrDefault(emptyList())
    }

    private fun getJsonArray(url: String): JSONArray {
        val request = Request.Builder()
            .url(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Fluxa-App")
            .build()
        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) error("HTTP ${response.code}")
            return JSONArray(response.body.string())
        }
    }

    private fun getJsonObject(url: String): JSONObject {
        val request = Request.Builder()
            .url(url)
            .header("Accept", "application/json")
            .header("User-Agent", "Fluxa-App")
            .build()
        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) error("HTTP ${response.code}")
            return JSONObject(response.body.string())
        }
    }

    private fun parseReleaseContributors(body: String): List<ContributorInfo> {
        if (body.isBlank()) return emptyList()
        val regex = Regex("(?<![A-Za-z0-9_])@([A-Za-z0-9-]+)")
        val counts = linkedMapOf<String, Int>()
        regex.findAll(body).forEach { match ->
            val login = match.groupValues[1]
            if (!login.equals("username", ignoreCase = true)) counts[login] = (counts[login] ?: 0) + 1
        }
        return counts.map { (login, count) ->
            ContributorInfo(
                login = login,
                avatarUrl = "https://github.com/$login.png?size=96",
                profileUrl = "https://github.com/$login",
                contributions = count,
            )
        }
    }

    suspend fun checkUpdate(): UpdateInfo? {
        val latest = fetchLatestRelease() ?: return null
        if (!isNewerVersion(latest.versionName, BuildConfig.VERSION_NAME)) {
            Log.i("UpdateManager", "Already up to date")
            return null
        }
        if (latest.url.isBlank()) {
            Log.w("UpdateManager", "Newer release has no compatible APK asset")
            return null
        }
        return latest
    }

    fun isUpdateAvailable(update: UpdateInfo): Boolean = isNewerVersion(update.versionName, BuildConfig.VERSION_NAME)

    private fun findMatchingApk(apkAssets: List<JSONObject>): String? {
        val candidates = apkAssets.filter {
            it.getString("name").contains(BuildConfig.DEVICE_FLAVOR, ignoreCase = true)
        }
        if (candidates.isEmpty()) return null

        for (abi in Build.SUPPORTED_ABIS) {
            val match = candidates.find { it.getString("name").contains(abi, ignoreCase = true) }
            if (match != null) return match.getString("browser_download_url")
        }

        return candidates.find { it.getString("name").contains("universal", ignoreCase = true) }
            ?.getString("browser_download_url")
    }

    private fun isNewerVersion(remote: String, current: String): Boolean {
        return FluxaCoreNative.versionIsNewer(remote, current)
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
