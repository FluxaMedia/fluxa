package com.fluxa.app.data.repository

import java.net.InetAddress
import com.fluxa.app.core.rust.FluxaCoreNative

object PluginNetGuard {

    fun isBlockedAddress(address: InetAddress): Boolean =
        !FluxaCoreNative.pluginNetworkAddressAllowed(address.hostAddress.orEmpty())

    fun resolveAllowedAddresses(host: String): List<InetAddress>? {
        val addresses = try {
            InetAddress.getAllByName(host)
        } catch (_: Exception) {
            return null
        }
        if (addresses.isEmpty() || addresses.any(::isBlockedAddress)) return null
        return addresses.toList()
    }

}
