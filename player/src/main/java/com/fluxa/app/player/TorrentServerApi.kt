package com.fluxa.app.player

import com.fluxa.app.common.Constants
import com.google.gson.annotations.SerializedName
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.Body
import retrofit2.http.POST
data class TorrentRequest(
    val action: String,
    val link: String? = null,
    val hash: String? = null,
    val title: String? = null,
    @SerializedName("save_to_db") val saveToDb: Boolean = false,
    @SerializedName("file_id") val fileId: Int? = null
)

data class TorrentSettings(
    @SerializedName("PreloadSize") val preloadSize: Long = 16,
)

data class TorrentAddRequest(
    val link: String,
    val title: String = "",
    val poster: String = "",
    @SerializedName("save_to_db") val saveToDb: Boolean = false
)

interface TorrentServerApi {
    //  Matrix / Lana (Modern) @POST("torrents")
    @POST("torrents")
    suspend fun addTorrent(@Body request: TorrentRequest): Response<TorrentStatus>

    @POST("torrents")
    suspend fun getTorrent(@Body request: TorrentRequest): Response<TorrentStatus>

    @POST("torrents")
    suspend fun removeTorrent(@Body request: TorrentRequest): Response<Unit>

    //  Legacy (Classic) @POST("torrent/add")
    suspend fun addTorrentLegacy(@Body request: TorrentAddRequest): Response<TorrentStatus>

    @POST("torrent/get")
    suspend fun getTorrentLegacy(@Body request: String): Response<TorrentStatus> // Often expects raw hash string or simple JSON

    @POST("settings")
    suspend fun updateSettings(@Body request: TorrentSettings): Response<Unit>

    companion object {
        private val localClient by lazy {
            okhttp3.OkHttpClient.Builder()
                .connectTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
                .readTimeout(90, java.util.concurrent.TimeUnit.SECONDS) // metadata fetch can take 30-60s
                .writeTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
                .retryOnConnectionFailure(true)
                .build()
        }

        fun create(baseUrl: String = Constants.LocalServer.TORRENT_SERVER_BASE_URL): TorrentServerApi {
            val url = if (baseUrl.endsWith("/")) baseUrl else "$baseUrl/"
            return Retrofit.Builder()
                .baseUrl(url)
                .client(localClient)
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(TorrentServerApi::class.java)
        }
    }
}
