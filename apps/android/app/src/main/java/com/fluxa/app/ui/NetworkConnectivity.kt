package com.fluxa.app.ui

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged

internal fun Context.isNetworkAvailableNow(): Boolean {
    val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return false
    val network = connectivityManager.activeNetwork ?: return false
    val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return false
    return capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
        capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
}

internal fun Context.observeNetworkAvailable(): Flow<Boolean> = callbackFlow {
    val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
    if (connectivityManager == null) {
        trySend(false)
        close()
        return@callbackFlow
    }

    fun sendCurrentState() {
        trySend(isNetworkAvailableNow())
    }

    val callback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) {
            sendCurrentState()
        }

        override fun onLost(network: Network) {
            sendCurrentState()
        }

        override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
            sendCurrentState()
        }
    }

    sendCurrentState()
    connectivityManager.registerDefaultNetworkCallback(callback)
    awaitClose {
        runCatching { connectivityManager.unregisterNetworkCallback(callback) }
    }
}.distinctUntilChanged()
