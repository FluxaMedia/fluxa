package com.fluxa.app.player

import com.fluxa.app.common.Constants
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.Body
import retrofit2.http.POST
import com.google.gson.annotations.SerializedName

data class TorrRequest(
    val action: String,
    val link: String? = null,
    val hash: String? = null,
    val title: String? = null,
    @SerializedName("save_to_db") val saveToDb: Boolean = false,
    @SerializedName("file_id") val fileId: Int? = null
)

data class TorrStatus(
    @SerializedName("hash") val hash: String,
    @SerializedName("title") val title: String,
    @SerializedName("download_speed") val downloadSpeed: Double,
    @SerializedName("active_peers") val activePeers: Int,
    @SerializedName("total_peers") val totalPeers: Int,
    @SerializedName("progress") val progress: Double,
    @SerializedName("stat") val stat: Int = 0, // 0: Metadata, 1: Preload, 2: Down, 3: Play
    @SerializedName("stat_string") val statString: String = "",
    @SerializedName("preload") val preload: Int = 0,
    @SerializedName("loaded_size") val loadedSize: Long = 0, //  Real bytes loaded for buffer
    @SerializedName("preload_size") val preloadSize: Long = 0, //  Target buffer size
    @SerializedName("file_stats") val fileStats: List<TorrFileStat>?
)

data class TorrFileStat(
    @SerializedName("id") val id: Int,
    @SerializedName("path") val path: String,
    @SerializedName("length") val length: Long
)

data class TorrSettings(
    @SerializedName("PreloadSize") val preloadSize: Long = 16,
)

data class TorrAddRequest(
    val link: String,
    val title: String = "",
    val poster: String = "",
    @SerializedName("save_to_db") val saveToDb: Boolean = false
)

interface TorrServerApi {
    //  Matrix / Lana (Modern) @POST("torrents")
    @POST("torrents")
    suspend fun addTorrent(@Body request: TorrRequest): Response<TorrStatus>

    @POST("torrents")
    suspend fun getTorrent(@Body request: TorrRequest): Response<TorrStatus>

    @POST("torrents")
    suspend fun removeTorrent(@Body request: TorrRequest): Response<Unit>

    //  Legacy (Classic) @POST("torrent/add")
    suspend fun addTorrentLegacy(@Body request: TorrAddRequest): Response<TorrStatus>

    @POST("torrent/get")
    suspend fun getTorrentLegacy(@Body request: String): Response<TorrStatus> // Often expects raw hash string or simple JSON

    @POST("settings")
    suspend fun updateSettings(@Body request: TorrSettings): Response<Unit>

    companion object {
        private val localClient by lazy {
            okhttp3.OkHttpClient.Builder()
                .connectTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
                .readTimeout(90, java.util.concurrent.TimeUnit.SECONDS) // metadata 30-60s sürebilir
                .writeTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
                .retryOnConnectionFailure(true)
                .build()
        }

        fun create(baseUrl: String = Constants.LocalServer.TORR_SERVER_BASE_URL): TorrServerApi {
            val url = if (baseUrl.endsWith("/")) baseUrl else "$baseUrl/"
            return Retrofit.Builder()
                .baseUrl(url)
                .client(localClient)
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(TorrServerApi::class.java)
        }
    }
}
