package com.fluxa.app.desktop.auth

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.remote.NuvioService
import java.util.concurrent.TimeUnit
import okhttp3.OkHttpClient
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory

fun buildDesktopNuvioService(): NuvioService {
    val client = OkHttpClient.Builder()
        .addInterceptor { chain ->
            chain.proceed(
                chain.request().newBuilder()
                    .header("apikey", PlatformSecrets.nuvioSupabaseKey)
                    .header("Content-Type", "application/json")
                    .build()
            )
        }
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(20, TimeUnit.SECONDS)
        .build()
    return Retrofit.Builder()
        .baseUrl(PlatformSecrets.nuvioSupabaseUrl)
        .client(client)
        .addConverterFactory(GsonConverterFactory.create())
        .build()
        .create(NuvioService::class.java)
}
