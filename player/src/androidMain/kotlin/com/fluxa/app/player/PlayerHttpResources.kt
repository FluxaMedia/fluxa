package com.fluxa.app.player

import okhttp3.ConnectionPool
import okhttp3.Dispatcher
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit

object PlayerHttpResources {
    private val connectionPool = ConnectionPool(15, 5, TimeUnit.MINUTES)
    private val dispatcher = Dispatcher().apply {
        maxRequests = 32
        maxRequestsPerHost = 8
    }

    fun newBuilder(): OkHttpClient.Builder = OkHttpClient.Builder()
        .connectionPool(connectionPool)
        .dispatcher(dispatcher)
}
