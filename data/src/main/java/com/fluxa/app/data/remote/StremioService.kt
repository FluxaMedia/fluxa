package com.fluxa.app.data.remote

import com.fluxa.app.data.repository.HttpRequestSecurity
import okhttp3.Cookie
import okhttp3.CookieJar
import okhttp3.HttpUrl
import okhttp3.OkHttpClient
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Url
import java.util.concurrent.TimeUnit

interface StremioService {
    @POST("api/login")
    suspend fun login(@Body request: LoginRequest): Response<AuthResponse>

    @POST("api/register")
    suspend fun register(@Body request: LoginRequest): Response<AuthResponse>

    @POST("api/addonCollectionGet")
    suspend fun getAddons(@Body request: AuthRequest): Response<AddonCollectionResponse>

    @POST("api/datastorePut")
    suspend fun datastorePut(@Body request: DatastorePutRequest): Response<Any>

    @POST("api/datastoreGet")
    suspend fun getDatastore(@Body request: DatastoreRequest): Response<DatastoreResponse>

    @GET
    suspend fun getUserAddons(@Url url: String): List<AddonDescriptor>

    @GET("catalog/{type}/{id}.json")
    suspend fun getCatalog(@Path("type") type: String, @Path("id") id: String): CatalogResponse

    @GET("catalog/{type}/{id}/skip={skip}.json")
    suspend fun getCatalogWithSkip(@Path("type") type: String, @Path("id") id: String, @Path("skip") skip: Int): CatalogResponse

    @GET("catalog/{type}/{id}/genre={genre}.json")
    suspend fun getCatalogWithGenre(@Path("type") type: String, @Path("id") id: String, @Path("genre") genre: String): CatalogResponse

    @GET("meta/{type}/{id}.json")
    suspend fun getMetaDetail(@Path("type") type: String, @Path("id") id: String): MetaDetailResponse

    @GET
    suspend fun getStreams(@Url url: String): StreamResponse

    companion object {
        private const val API_URL = "https://api.strem.io/"

        val sharedClient: OkHttpClient by lazy {
            createClient(withJsonHeader = true)
        }

        val scrapperClient: OkHttpClient by lazy {
            createClient(withJsonHeader = false)
        }

        private fun createClient(withJsonHeader: Boolean): OkHttpClient {
            val logging = okhttp3.logging.HttpLoggingInterceptor().apply {
                level = okhttp3.logging.HttpLoggingInterceptor.Level.NONE
            }

            val ipv4OnlyDns = object : okhttp3.Dns {
                override fun lookup(hostname: String): List<java.net.InetAddress> {
                    return okhttp3.Dns.SYSTEM.lookup(hostname).filter { it is java.net.Inet4Address }
                }
            }
            
            val cookieJar = if (!withJsonHeader) {
                object : CookieJar {
                    private val cookieStore = mutableMapOf<String, List<Cookie>>()
                    override fun saveFromResponse(url: HttpUrl, cookies: List<Cookie>) {
                        cookieStore[url.host] = cookies
                    }
                    override fun loadForRequest(url: HttpUrl): List<Cookie> {
                        return cookieStore[url.host] ?: listOf()
                    }
                }
            } else CookieJar.NO_COOKIES

            return OkHttpClient.Builder()
                .dns(ipv4OnlyDns)
                .cookieJar(cookieJar)
                .addInterceptor { chain ->
                    chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
                }
                .addInterceptor { chain ->
                    val builder = chain.request().newBuilder()
                        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
                    if (withJsonHeader) {
                        builder.header("Accept", "application/json")
                    }
                    chain.proceed(builder.build())
                }
                .addInterceptor(logging)
                .connectTimeout(10, TimeUnit.SECONDS)
                .readTimeout(10, TimeUnit.SECONDS)
                .writeTimeout(10, TimeUnit.SECONDS)
                .retryOnConnectionFailure(true)
                .build()
        }

        fun create(): StremioService {
            return Retrofit.Builder()
                .baseUrl(API_URL)
                .callFactory { request -> sharedClient.newCall(request) }
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(StremioService::class.java)
        }

        private var dynamicInstance: StremioService? = null
        fun createDynamic(): StremioService {
            if (dynamicInstance == null) {
                dynamicInstance = create()
            }
            return dynamicInstance!!
        }

        fun createAuth(): StremioService {
            return Retrofit.Builder()
                .baseUrl(API_URL)
                .callFactory { request -> sharedClient.newCall(request) }
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(StremioService::class.java)
        }
    }
}
