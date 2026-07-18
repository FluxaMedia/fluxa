package com.fluxa.app.ui.catalog

import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.TransferListener
import androidx.media3.datasource.okhttp.OkHttpDataSource
import okhttp3.OkHttpClient
import okhttp3.Dns
import java.net.Inet4Address

internal object TrailerResolver {

    private val httpClient = OkHttpClient.Builder()
        .dns { hostname ->
            val addresses = Dns.SYSTEM.lookup(hostname)
            addresses.filterIsInstance<Inet4Address>().ifEmpty { addresses }
        }
        .build()

    fun init(cacheDir: java.io.File) = Unit

    fun mediaDataSourceFactory(): DataSource.Factory {
        val upstream = OkHttpDataSource.Factory(httpClient)
            .setUserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        return DataSource.Factory { TrailerRangeDataSource(upstream.createDataSource()) }
    }
}

private class TrailerRangeDataSource(
    private val delegate: DataSource
) : DataSource {
    override fun addTransferListener(transferListener: TransferListener) {
        delegate.addTransferListener(transferListener)
    }

    override fun open(dataSpec: DataSpec): Long {
        val headers = dataSpec.httpRequestHeaders
        val request = if (headers.keys.any { it.equals("Range", ignoreCase = true) }) {
            dataSpec
        } else {
            dataSpec.withRequestHeaders(headers + ("Range" to "bytes=0-"))
        }
        return delegate.open(request)
    }

    override fun read(buffer: ByteArray, offset: Int, length: Int): Int = delegate.read(buffer, offset, length)

    override fun getUri() = delegate.uri

    override fun getResponseHeaders() = delegate.responseHeaders

    override fun close() = delegate.close()
}
