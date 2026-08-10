package com.fluxa.app.ui.catalog

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import com.fluxa.app.player.ExternalSubtitleTrack

internal object AndroidExternalPlayerLauncher {
    fun launch(
        context: Context,
        url: String,
        title: String?,
        positionMs: Long,
        packageName: String?,
        headers: Map<String, String> = emptyMap(),
        subtitles: List<ExternalSubtitleTrack> = emptyList(),
    ): Boolean {
        val intent = createLaunchIntent(
            context = context,
            url = url,
            title = title,
            positionMs = positionMs,
            packageName = packageName,
            headers = headers,
            subtitles = subtitles,
        ) ?: return false
        return try {
            context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            true
        } catch (_: ActivityNotFoundException) {
            false
        }
    }

    fun createLaunchIntent(
        context: Context,
        url: String,
        title: String?,
        positionMs: Long,
        packageName: String?,
        headers: Map<String, String> = emptyMap(),
        subtitles: List<ExternalSubtitleTrack> = emptyList(),
    ): Intent? {
        val baseIntent = buildIntent(url, title, positionMs, headers, subtitles)
        val requestedPackage = packageName?.trim()?.takeIf { it.isNotEmpty() }
        if (requestedPackage != null) {
            val targeted = Intent(baseIntent).setPackage(requestedPackage)
            if (targeted.resolveActivity(context.packageManager) != null) return targeted
        }

        if (baseIntent.resolveActivity(context.packageManager) == null) {
            return Intent(Intent.ACTION_VIEW, Uri.parse(url)).takeIf {
                it.resolveActivity(context.packageManager) != null
            }
        }
        return Intent.createChooser(baseIntent, title ?: "Open with")
    }

    private fun buildIntent(
        url: String,
        title: String?,
        positionMs: Long,
        headers: Map<String, String>,
        subtitles: List<ExternalSubtitleTrack>,
    ): Intent = Intent(Intent.ACTION_VIEW).apply {
        setDataAndType(Uri.parse(url), "video/*")
        addCategory(Intent.CATEGORY_DEFAULT)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        title?.takeIf { it.isNotBlank() }?.let {
            putExtra(Intent.EXTRA_TITLE, it)
            putExtra("title", it)
            putExtra("media_title", it)
        }
        putExtra("position", positionMs.coerceAtLeast(0L))
        putExtra("position_ms", positionMs.coerceAtLeast(0L))
        putExtra("from_start", positionMs <= 0L)

        if (headers.isNotEmpty()) {
            val bundle = Bundle().apply {
                headers.forEach { (key, value) -> putString(key, value) }
            }
            // Different Android players read different conventional header extras.
            putExtra("headers", bundle)
            putExtra("com.android.browser.headers", bundle)
        }

        if (subtitles.isNotEmpty()) {
            val subtitleUris = subtitles.map { Uri.parse(it.url) }.toTypedArray()
            val subtitleNames = subtitles.map { track ->
                track.label?.takeIf(String::isNotBlank)
                    ?: track.language?.takeIf(String::isNotBlank)
                    ?: "Subtitle"
            }.toTypedArray()
            // VLC, mpv-android and several Android players recognize one or more of these
            // de-facto intent extras. Embedded tracks remain part of the media container.
            putExtra("subtitles_location", subtitles.first().url)
            putExtra("subs", subtitleUris)
            putExtra("subs.name", subtitleNames)
            putExtra("subtitle", subtitles.first().url)
        }
    }
}
