package com.fluxa.app.player

import android.app.Activity
import com.discord.socialsdk.DiscordSocialSdkInit

object DiscordPresenceNative {
    private const val applicationId = 1518004842860122174L
    private var loaded = false

    fun initialize(activity: Activity) {
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
        updateNative(title, "$episodeLine · $status", status, positionMs, durationMs, artworkUrl)
    }

    fun clear() = clearNative()

    private external fun initializeNative(applicationId: Long): Boolean
    private external fun authorizeNative(): Boolean
    private external fun updateNative(title: String, episodeLine: String, status: String, positionMs: Long, durationMs: Long, artworkUrl: String?): Boolean
    private external fun clearNative()
}
