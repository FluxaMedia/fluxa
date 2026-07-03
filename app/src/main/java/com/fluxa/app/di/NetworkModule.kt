package com.fluxa.app.di

import com.fluxa.app.BuildConfig
import android.net.http.X509TrustManagerExtensions
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.HttpRequestSecurity
import com.google.gson.Gson
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.Cookie
import okhttp3.CookieJar
import okhttp3.HttpUrl
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.util.concurrent.TimeUnit
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLEngine
import javax.net.ssl.SSLSocket
import javax.net.ssl.TrustManager
import javax.net.ssl.TrustManagerFactory
import javax.net.ssl.X509ExtendedTrustManager
import javax.net.ssl.X509TrustManager
import javax.inject.Named
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {

    @Provides
    @Singleton
    fun provideGson(): Gson = Gson()

    @Provides
    @Singleton
    fun provideLoggingInterceptor(): HttpLoggingInterceptor {
        return HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.NONE
        }
    }

    @Provides
    @Singleton
    fun provideConnectionPool(): okhttp3.ConnectionPool = okhttp3.ConnectionPool(
        maxIdleConnections = 15,
        keepAliveDuration = 5,
        timeUnit = TimeUnit.MINUTES
    )

    @Provides
    @Singleton
    fun provideDispatcher(): okhttp3.Dispatcher = okhttp3.Dispatcher().apply {
        maxRequests = 64
        maxRequestsPerHost = 16
    }

    @Provides
    @Singleton
    @Named("StremioClient")
    fun provideStremioOkHttpClient(logging: HttpLoggingInterceptor, connectionPool: okhttp3.ConnectionPool, dispatcher: okhttp3.Dispatcher): OkHttpClient {
        val ipv4OnlyDns = object : okhttp3.Dns {
            override fun lookup(hostname: String): List<java.net.InetAddress> {
                return okhttp3.Dns.SYSTEM.lookup(hostname).filter { it is java.net.Inet4Address }
            }
        }

        return OkHttpClient.Builder()
            .dns(ipv4OnlyDns)
            .connectionPool(connectionPool)
            .dispatcher(dispatcher)
            .addInterceptor { chain ->
                chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
            }
            .addInterceptor { chain ->
                val request = chain.request().newBuilder()
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
                    .header("Accept", "application/json")
                    .build()
                chain.proceed(request)
            }
            .addInterceptor(logging)
            .connectTimeout(10, TimeUnit.SECONDS)
            .readTimeout(10, TimeUnit.SECONDS)
            .writeTimeout(10, TimeUnit.SECONDS)
            .retryOnConnectionFailure(true)
            .build()
    }

    @Provides
    @Singleton
    @Named("GenericClient")
    fun provideGenericOkHttpClient(logging: HttpLoggingInterceptor, connectionPool: okhttp3.ConnectionPool, dispatcher: okhttp3.Dispatcher): OkHttpClient {
        return OkHttpClient.Builder()
            .connectionPool(connectionPool)
            .dispatcher(dispatcher)
            .addInterceptor { chain ->
                chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
            }
            .addInterceptor { chain ->
                val request = chain.request().newBuilder()
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
                    .build()
                chain.proceed(request)
            }
            .addInterceptor(logging)
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(15, TimeUnit.SECONDS)
            .build()
    }

    @Provides
    @Singleton
    @Named("AddonResourceClient")
    fun provideAddonResourceOkHttpClient(@Named("StremioClient") baseClient: OkHttpClient): OkHttpClient {
        val trustManager = PlatformChainTrustManager(defaultX509TrustManager())
        val sslContext = SSLContext.getInstance("TLS").apply {
            init(null, arrayOf<TrustManager>(trustManager), null)
        }
        return baseClient.newBuilder()
            .sslSocketFactory(sslContext.socketFactory, trustManager)
            .build()
    }

    @Provides
    @Singleton
    @Named("ImageClient")
    fun provideImageOkHttpClient(connectionPool: okhttp3.ConnectionPool): OkHttpClient {
        val imageDispatcher = okhttp3.Dispatcher().apply {
            maxRequests = 32
            maxRequestsPerHost = 10
        }
        return OkHttpClient.Builder()
            .connectionPool(connectionPool)
            .dispatcher(imageDispatcher)
            .addInterceptor { chain ->
                chain.proceed(
                    chain.request().newBuilder()
                        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
                        .build()
                )
            }
            .connectTimeout(8, TimeUnit.SECONDS)
            .readTimeout(12, TimeUnit.SECONDS)
            .build()
    }

    @Provides
    @Singleton
    fun provideStremioService(@Named("StremioClient") client: OkHttpClient): StremioService {
        return Retrofit.Builder()
            .baseUrl("https://api.strem.io/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(StremioService::class.java)
    }

    @Provides
    @Singleton
    fun provideTraktApi(@Named("GenericClient") client: OkHttpClient): TraktApi {
        return Retrofit.Builder()
            .baseUrl("https://api.trakt.tv/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(TraktApi::class.java)
    }

    @Provides
    @Singleton
    fun provideTmdbService(@Named("StremioClient") client: OkHttpClient): TmdbService {
        return Retrofit.Builder()
            .baseUrl("https://api.themoviedb.org/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(TmdbService::class.java)
    }

    @Provides
    @Singleton
    fun provideImdbApiService(@Named("StremioClient") client: OkHttpClient): ImdbApiService {
        return Retrofit.Builder()
            .baseUrl("https://api.imdbapi.dev/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(ImdbApiService::class.java)
    }

    @Provides
    @Singleton
    fun provideIntroDbService(@Named("StremioClient") client: OkHttpClient): IntroDbService {
        return Retrofit.Builder()
            .baseUrl("https://api.introdb.app/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(IntroDbService::class.java)
    }

    @Provides
    @Singleton
    fun provideAniSkipService(@Named("StremioClient") client: OkHttpClient): AniSkipService {
        return Retrofit.Builder()
            .baseUrl("https://api.aniskip.com/v2/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(AniSkipService::class.java)
    }

    @Provides
    @Singleton
    @Named("NuvioClient")
    fun provideNuvioOkHttpClient(logging: HttpLoggingInterceptor, connectionPool: okhttp3.ConnectionPool): OkHttpClient {
        return OkHttpClient.Builder()
            .connectionPool(connectionPool)
            .addInterceptor { chain ->
                chain.proceed(
                    chain.request().newBuilder()
                        .header("apikey", BuildConfig.NUVIO_SUPABASE_KEY)
                        .header("Content-Type", "application/json")
                        .build()
                )
            }
            .addInterceptor(logging)
            .connectTimeout(20, TimeUnit.SECONDS)
            .readTimeout(20, TimeUnit.SECONDS)
            .build()
    }

    @Provides
    @Singleton
    fun provideNuvioService(@Named("NuvioClient") client: OkHttpClient): NuvioService {
        return Retrofit.Builder()
            .baseUrl(BuildConfig.NUVIO_SUPABASE_URL)
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(NuvioService::class.java)
    }

    @Provides
    @Singleton
    fun provideNuvioAccountImportCoordinator(
        nuvioService: NuvioService,
        profileManager: com.fluxa.app.data.local.ProfileManager,
        watchlistManager: com.fluxa.app.data.local.WatchlistManager,
        addonRepository: com.fluxa.app.data.repository.AddonRepository
    ): com.fluxa.app.data.repository.NuvioAccountImportCoordinator {
        return com.fluxa.app.data.repository.NuvioAccountImportCoordinator(
            nuvioService = nuvioService,
            profileManager = profileManager,
            watchlistManager = watchlistManager,
            addonRepository = addonRepository,
            supabaseUrl = BuildConfig.NUVIO_SUPABASE_URL
        )
    }

    private fun defaultX509TrustManager(): X509TrustManager {
        val factory = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm())
        factory.init(null as java.security.KeyStore?)
        return factory.trustManagers
            .filterIsInstance<X509TrustManager>()
            .single()
    }

    private class PlatformChainTrustManager(
        private val delegate: X509TrustManager
    ) : X509ExtendedTrustManager() {
        private val extensions = X509TrustManagerExtensions(delegate)

        override fun checkClientTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) {
            delegate.checkClientTrusted(chain, authType)
        }

        override fun checkServerTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) {
            delegate.checkServerTrusted(chain, authType)
        }

        override fun checkClientTrusted(
            chain: Array<java.security.cert.X509Certificate>,
            authType: String,
            socket: java.net.Socket
        ) {
            delegate.checkClientTrusted(chain, authType)
        }

        override fun checkServerTrusted(
            chain: Array<java.security.cert.X509Certificate>,
            authType: String,
            socket: java.net.Socket
        ) {
            extensions.checkServerTrusted(chain, authType, peerHost(socket))
        }

        override fun checkClientTrusted(
            chain: Array<java.security.cert.X509Certificate>,
            authType: String,
            engine: SSLEngine
        ) {
            delegate.checkClientTrusted(chain, authType)
        }

        override fun checkServerTrusted(
            chain: Array<java.security.cert.X509Certificate>,
            authType: String,
            engine: SSLEngine
        ) {
            extensions.checkServerTrusted(chain, authType, engine.peerHost)
        }

        override fun getAcceptedIssuers(): Array<java.security.cert.X509Certificate> = delegate.acceptedIssuers

        private fun peerHost(socket: java.net.Socket): String {
            return (socket as? SSLSocket)?.handshakeSession?.peerHost
                ?: socket.inetAddress.hostName
        }
    }
}
