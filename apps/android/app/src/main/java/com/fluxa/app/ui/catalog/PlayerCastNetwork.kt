package com.fluxa.app.ui.catalog

import android.net.Uri
import com.fluxa.app.player.TorrentServerEngine
import java.net.Inet4Address
import java.net.NetworkInterface

internal object PlayerCastNetwork {
    fun reachableUrl(rawUrl: String): String {
        val uri = runCatching { Uri.parse(rawUrl) }.getOrNull() ?: return rawUrl
        val host = uri.host ?: return rawUrl
        if (uri.scheme != "http" || host !in LOOPBACK_HOSTS) return rawUrl

        val lanIp = deviceLanIpv4() ?: return rawUrl
        val token = TorrentServerEngine.castAccessToken
        val builder = uri.buildUpon()
            .authority(if (uri.port > 0) "$lanIp:${uri.port}" else lanIp)
        if (token.isNotBlank() && uri.getQueryParameter(ACCESS_TOKEN_QUERY).isNullOrBlank()) {
            builder.appendQueryParameter(ACCESS_TOKEN_QUERY, token)
        }
        return builder.build().toString()
    }

    private fun deviceLanIpv4(): String? = runCatching {
        NetworkInterface.getNetworkInterfaces().asSequence()
            .filter { network -> network.isUp && !network.isLoopback }
            .flatMap { network -> network.inetAddresses.asSequence() }
            .filterIsInstance<Inet4Address>()
            .mapNotNull { address -> address.hostAddress }
            .firstOrNull { address ->
                !address.startsWith("127.") &&
                    !address.startsWith("169.254.") &&
                    address != "0.0.0.0"
            }
    }.getOrNull()

    private const val ACCESS_TOKEN_QUERY = "access_token"
    private val LOOPBACK_HOSTS = setOf("127.0.0.1", "localhost", "::1")
}
