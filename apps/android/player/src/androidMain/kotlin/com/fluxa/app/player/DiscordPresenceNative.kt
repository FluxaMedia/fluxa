package com.fluxa.app.player

import android.app.Activity
import com.discord.socialsdk.DiscordSocialSdkInit

object DiscordPresenceNative {
    private const val applicationId = 1518004842860122174L
    private const val discordPackage = "com.discord"
    private var loaded = false
    private var enabled = false

    fun initialize(activity: Activity, enabled: Boolean) {
        val shouldEnable = enabled && isDiscordInstalled(activity)
        if (!shouldEnable) {
            if (this.enabled && loaded) clearNative()
            this.enabled = false
            return
        }
        this.enabled = true
        if (!loaded) {
            System.loadLibrary("fluxa_libass_renderer")
            loaded = true
        }
        DiscordSocialSdkInit.setEngineActivity(activity)
        initializeNative(applicationId)
        authorizeNative()
    }

    fun update(
        title: String,
        episodeLine: String,
        status: String,
        positionMs: Long,
        durationMs: Long,
        artworkUrl: String?
    ) {
        if (!enabled) return
        updateNative(title, "$episodeLine · $status", status, positionMs, durationMs, artworkUrl)
    }

    fun clear() {
        if (enabled) clearNative()
    }

    private fun isDiscordInstalled(activity: Activity): Boolean = runCatching {
        activity.packageManager.getApplicationInfo(discordPackage, 0).enabled
    }.getOrDefault(false)

    private external fun initializeNative(applicationId: Long): Boolean
    private external fun authorizeNative(): Boolean
    private external fun updateNative(title: String, episodeLine: String, status: String, positionMs: Long, durationMs: Long, artworkUrl: String?): Boolean
    private external fun clearNative()
}
