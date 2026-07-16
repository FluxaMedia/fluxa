package com.fluxa.app.data.remote

import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.DELETE
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.PATCH
import retrofit2.http.POST
import retrofit2.http.Query

interface NuvioService {
    @GET("functions/v1/health-check")
    suspend fun healthCheck(): Response<NuvioHealth>

    @POST("auth/v1/signup")
    suspend fun signUp(@Body credentials: NuvioCredentials): Response<NuvioSessionDto>

    @POST("auth/v1/token")
    suspend fun signIn(@Query("grant_type") grantType: String = "password", @Body credentials: NuvioCredentials): Response<NuvioSessionDto>

    @POST("auth/v1/token")
    suspend fun refreshToken(@Query("grant_type") grantType: String = "refresh_token", @Body request: NuvioRefreshRequestDto): Response<NuvioSessionDto>

    @GET("auth/v1/user")
    suspend fun getUser(@Header("Authorization") authorization: String): Response<NuvioUser>

    @POST("rest/v1/rpc/sync_pull_profiles")
    suspend fun pullProfiles(@Header("Authorization") authorization: String, @Body body: Map<String, String> = emptyMap()): Response<List<NuvioProfileDto>>

    @GET("rest/v1/addons")
    suspend fun pullAddons(
        @Header("Authorization") authorization: String,
        @Query("select") select: String = "*",
        @Query("profile_id") profileId: String,
        @Query("order") order: String = "sort_order"
    ): Response<List<NuvioAddonDto>>

    @POST("rest/v1/rpc/sync_push_profiles")
    suspend fun pushProfiles(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_push_addons")
    suspend fun pushAddons(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @PATCH("rest/v1/addons")
    suspend fun updateAddon(
        @Header("Authorization") authorization: String,
        @Query("id") id: String,
        @Query("profile_id") profileId: String,
        @Body body: Map<String, @JvmSuppressWildcards Any?>
    ): Response<Unit>

    @DELETE("rest/v1/addons")
    suspend fun deleteAddon(
        @Header("Authorization") authorization: String,
        @Query("id") id: String,
        @Query("profile_id") profileId: String
    ): Response<Unit>

    @POST("rest/v1/rpc/sync_push_library")
    suspend fun pushLibrary(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_push_watch_progress")
    suspend fun pushWatchProgress(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_delete_watch_progress")
    suspend fun deleteWatchProgress(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_push_watched_items")
    suspend fun pushWatchedItems(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_delete_watched_items")
    suspend fun deleteWatchedItems(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_push_collections")
    suspend fun pushCollections(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_push_profile_settings_blob")
    suspend fun pushProfileSettings(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any?>): Response<Unit>

    @POST("rest/v1/rpc/sync_pull_library")
    suspend fun pullLibrary(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any>): Response<List<NuvioLibraryItemDto>>

    @POST("rest/v1/rpc/sync_pull_watch_progress")
    suspend fun pullWatchProgress(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any>): Response<List<NuvioWatchProgressDto>>

    @POST("rest/v1/rpc/sync_pull_watched_items")
    suspend fun pullWatchedItems(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any>): Response<List<NuvioWatchedItemDto>>

    @POST("rest/v1/rpc/sync_pull_collections")
    suspend fun pullCollections(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any>): Response<List<NuvioCollectionRowDto>>

    @POST("rest/v1/rpc/sync_pull_profile_settings_blob")
    suspend fun pullProfileSettings(@Header("Authorization") authorization: String, @Body body: Map<String, @JvmSuppressWildcards Any>): Response<List<NuvioProfileSettingsRowDto>>

    @POST("rest/v1/rpc/get_avatar_catalog")
    suspend fun listAvatars(@Body body: Map<String, String> = emptyMap()): Response<List<NuvioAvatarDto>>
}
