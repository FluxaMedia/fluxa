package com.fluxa.app.ui

import android.content.Intent
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow

internal class OAuthRedirectHandler {
    private val traktCodes = MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val simklCodes = MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val anilistCodes = MutableSharedFlow<String>(extraBufferCapacity = 1)

    val trakt: SharedFlow<String> = traktCodes
    val simkl: SharedFlow<String> = simklCodes
    val anilist: SharedFlow<String> = anilistCodes

    fun handle(intent: Intent) {
        if (intent.action != Intent.ACTION_VIEW) return
        val data = intent.data ?: return
        if (data.scheme != "fluxa" || data.host != "oauth") return
        val code = data.getQueryParameter("code") ?: return
        when (data.lastPathSegment) {
            "trakt" -> traktCodes.tryEmit(code)
            "simkl" -> simklCodes.tryEmit(code)
            "anilist" -> anilistCodes.tryEmit(code)
        }
    }
}
