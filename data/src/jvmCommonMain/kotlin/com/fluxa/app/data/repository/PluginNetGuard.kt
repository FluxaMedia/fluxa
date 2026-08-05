package com.fluxa.app.data.repository

import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress

object PluginNetGuard {

    fun isSchemeAllowed(scheme: String?): Boolean =
        scheme.equals("http", ignoreCase = true) || scheme.equals("https", ignoreCase = true)

    fun isBlockedAddress(address: InetAddress): Boolean = when (address) {
        is Inet4Address -> isBlockedIpv4(address)
        is Inet6Address -> address.toIpv4Mapped()?.let(::isBlockedIpv4) ?: isBlockedIpv6(address)
        else -> true
    }

    fun resolveAllowedAddresses(host: String): List<InetAddress>? {
        val addresses = try {
            InetAddress.getAllByName(host)
        } catch (_: Exception) {
            return null
        }
        if (addresses.isEmpty() || addresses.any(::isBlockedAddress)) return null
        return addresses.toList()
    }

    private fun isBlockedIpv4(address: Inet4Address): Boolean {
        val octets = address.address.map { it.toInt() and 0xFF }
        return address.isLoopbackAddress ||
            address.isSiteLocalAddress ||
            address.isLinkLocalAddress ||
            address.isAnyLocalAddress ||
            octets == BROADCAST_OCTETS ||
            (octets[0] == 100 && octets[1] in 64..127)
    }

    private fun isBlockedIpv6(address: Inet6Address): Boolean {
        val bytes = address.address
        val firstSegment = ((bytes[0].toInt() and 0xFF) shl 8) or (bytes[1].toInt() and 0xFF)
        return address.isLoopbackAddress ||
            address.isAnyLocalAddress ||
            (firstSegment and 0xfe00) == 0xfc00 ||
            (firstSegment and 0xffc0) == 0xfe80
    }

    private fun Inet6Address.toIpv4Mapped(): Inet4Address? {
        val bytes = address
        val isMapped = (0..9).all { bytes[it].toInt() == 0 } &&
            bytes[10] == 0xFF.toByte() && bytes[11] == 0xFF.toByte()
        if (!isMapped) return null
        return InetAddress.getByAddress(byteArrayOf(bytes[12], bytes[13], bytes[14], bytes[15])) as Inet4Address
    }

    private val BROADCAST_OCTETS = listOf(255, 255, 255, 255)
}
