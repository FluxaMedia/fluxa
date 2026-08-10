package com.fluxa.app.ui

import android.content.Intent
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow

internal data class OAuthCodeRedirect(
    val code: String,
    val state: String? = null,
)

internal class OAuthRedirectHandler {
    private val traktCodes = MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val simklCodes = MutableSharedFlow<OAuthCodeRedirect>(extraBufferCapacity = 1)
    private val anilistTokens = MutableSharedFlow<String>(extraBufferCapacity = 1)

    val trakt: SharedFlow<String> = traktCodes
    val simkl: SharedFlow<OAuthCodeRedirect> = simklCodes
    val anilist: SharedFlow<String> = anilistTokens

    fun handle(intent: Intent) {
        if (intent.action != Intent.ACTION_VIEW) return
        val data = intent.data ?: return
        if (data.scheme != "fluxa" || data.host != "oauth") return
        when (data.lastPathSegment) {
            "trakt" -> data.getQueryParameter("code")?.let(traktCodes::tryEmit)
            "simkl" -> data.getQueryParameter("code")?.let { code ->
                simklCodes.tryEmit(OAuthCodeRedirect(code, data.getQueryParameter("state")))
            }
            "anilist" -> data.fragmentParameter("access_token")?.let(anilistTokens::tryEmit)
        }
    }
}

private fun android.net.Uri.fragmentParameter(name: String): String? =
    fragment
        ?.split('&')
        ?.asSequence()
        ?.mapNotNull { part ->
            val separator = part.indexOf('=')
            if (separator <= 0) null
            else android.net.Uri.decode(part.substring(0, separator)) to android.net.Uri.decode(part.substring(separator + 1))
        }
        ?.firstOrNull { (key, _) -> key == name }
        ?.second
        ?.takeIf { it.isNotBlank() }
