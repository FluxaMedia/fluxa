package com.fluxa.app.core.rust

import android.content.Context
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.OfflineSubtitleOption
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.google.gson.Gson

internal class AndroidOfflineEffectHandler(
    private val context: Context,
    private val gson: Gson
) {
    suspend fun enqueue(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val result = OfflineDownloadManager.getInstance(context).enqueue(
            profileId = payload.stringOrNull("profileId"),
            meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java),
            video = payload.objectValue("video")?.let { gson.fromJson(gson.toJsonTree(it), Video::class.java) },
            videoId = payload.stringOrNull("videoId"),
            stream = gson.fromJson(gson.toJsonTree(payload["stream"]), Stream::class.java),
            subtitle = payload.objectValue("subtitle")?.let {
                gson.fromJson(gson.toJsonTree(it), OfflineSubtitleOption::class.java)
            },
            profileLanguage = payload.stringOrNull("language")
        )
        return result.fold(
            onSuccess = { HeadlessEffectCompletion(effect.id, "ok", value = it) },
            onFailure = {
                HeadlessEffectCompletion(
                    effectId = effect.id,
                    status = "error",
                    error = mapOf("code" to (it.message ?: "offline_download_failed"))
                )
            }
        )
    }
}
