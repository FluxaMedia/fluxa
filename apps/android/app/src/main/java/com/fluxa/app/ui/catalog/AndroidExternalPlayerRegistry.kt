package com.fluxa.app.ui.catalog

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import com.fluxa.app.shared.feature.settings.SettingsChoiceOption

internal object AndroidExternalPlayerRegistry {
    fun installedVideoPlayers(context: Context): List<SettingsChoiceOption> {
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(Uri.parse("https://example.com/video.mp4"), "video/*")
            addCategory(Intent.CATEGORY_DEFAULT)
        }
        val packageManager = context.packageManager
        val resolved = if (Build.VERSION.SDK_INT >= 33) {
            packageManager.queryIntentActivities(intent, android.content.pm.PackageManager.ResolveInfoFlags.of(0L))
        } else {
            @Suppress("DEPRECATION")
            packageManager.queryIntentActivities(intent, 0)
        }
        return resolved
            .asSequence()
            .mapNotNull { info ->
                val packageName = info.activityInfo?.packageName?.takeIf { it != context.packageName } ?: return@mapNotNull null
                val label = runCatching { info.loadLabel(packageManager)?.toString() }.getOrNull()
                    ?.takeIf(String::isNotBlank)
                    ?: packageName
                SettingsChoiceOption(packageName, label, packageName)
            }
            .distinctBy { it.value }
            .sortedBy { it.label.lowercase() }
            .toList()
    }
}
