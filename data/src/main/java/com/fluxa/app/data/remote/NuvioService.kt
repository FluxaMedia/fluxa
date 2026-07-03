package com.fluxa.app.data.remote

import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.POST
import retrofit2.http.Query

interface NuvioService {
    @POST("auth/v1/signup")
    suspend fun signUp(@Body credentials: NuvioCredentials): Response<NuvioSession>

    @POST("auth/v1/token")
    suspend fun signIn(@Query("grant_type") grantType: String = "password", @Body credentials: NuvioCredentials): Response<NuvioSession>

    @POST("auth/v1/token")
    suspend fun refreshToken(@Query("grant_type") grantType: String = "refresh_token", @Body request: NuvioRefreshRequest): Response<NuvioSession>

    @GET("auth/v1/user")
    suspend fun getUser(@Header("Authorization") authorization: String): Response<NuvioUser>

    @POST("rest/v1/rpc/sync_pull_profiles")
    suspend fun pullProfiles(@Header("Authorization") authorization: String, @Body body: Map<String, String> = emptyMap()): Response<List<NuvioProfile>>

    @GET("rest/v1/addons")
    suspend fun pullAddons(
        @Header("Authorization") authorization: String,
        @Query("select") select: String = "*",
        @Query("profile_id") profileId: String,
        @Query("order") order: String = "sort_order"
    ): Response<List<NuvioAddon>>

    @POST("rest/v1/rpc/sync_pull_library")
    suspend fun pullLibrary(@Header("Authorization") authorization: String, @Body body: Map<String, Any>): Response<List<NuvioLibraryItem>>

    @POST("rest/v1/rpc/sync_pull_watch_progress")
    suspend fun pullWatchProgress(@Header("Authorization") authorization: String, @Body body: Map<String, Any>): Response<List<NuvioWatchProgress>>

    @POST("rest/v1/rpc/sync_pull_watched_items")
    suspend fun pullWatchedItems(@Header("Authorization") authorization: String, @Body body: Map<String, Any>): Response<List<NuvioWatchedItem>>

    @POST("rest/v1/rpc/sync_pull_collections")
    suspend fun pullCollections(@Header("Authorization") authorization: String, @Body body: Map<String, Any>): Response<List<NuvioCollectionRow>>

    @POST("rest/v1/rpc/sync_pull_profile_settings_blob")
    suspend fun pullProfileSettings(@Header("Authorization") authorization: String, @Body body: Map<String, Any>): Response<List<NuvioProfileSettingsRow>>

    @POST("rest/v1/rpc/get_avatar_catalog")
    suspend fun listAvatars(@Body body: Map<String, String> = emptyMap()): Response<List<NuvioAvatar>>
}
