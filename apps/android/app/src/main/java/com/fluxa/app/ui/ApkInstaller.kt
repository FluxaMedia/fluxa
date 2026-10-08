package com.fluxa.app.ui

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.FileProvider
import java.io.File

object ApkInstaller {
    fun install(activity: Activity, path: String) {
        if (Build.VERSION.SDK_INT >= 26 && !activity.packageManager.canRequestPackageInstalls()) {
            val settings = Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${activity.packageName}"))
            runCatching { activity.startActivity(settings) }
            return
        }
        val uri = FileProvider.getUriForFile(activity, "${activity.packageName}.updates", File(path))
        val intent = Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, "application/vnd.android.package-archive")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
        runCatching { activity.startActivity(intent) }
    }
}
