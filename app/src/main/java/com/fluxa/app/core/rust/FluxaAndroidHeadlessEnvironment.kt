@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import android.content.Context
import android.util.Log
import com.fluxa.app.core.StremioId
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import com.fluxa.app.domain.playback.PlaybackProgressScheduler
import com.fluxa.app.domain.playback.PlaybackSyncCoordinator
import com.fluxa.app.data.repository.HttpEffectExecutor
import com.fluxa.app.data.repository.MdblistRatingsClient
import com.fluxa.app.data.repository.library.ProviderAdapters
import com.fluxa.app.data.repository.library.ProviderContinueWatchingRepository
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.PluginRepositoryManager
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.OkHttpClient
import javax.inject.Named

data class StreamProgressUpdate(
    val requestId: String,
    val streams: List<Stream>,
    val completedAddonNames: List<String>,
    val loadingAddonNames: List<String>
)

@Singleton
class FluxaAndroidHeadlessEnvironment @Inject constructor(
    @param:ApplicationContext internal val context: Context,
    internal val repository: StremioRepository,
    internal val addonRepository: AddonRepository,
    internal val traktRepository: TraktRepository,
    internal val watchlistManager: WatchlistManager,
    internal val streamDiscovery: StreamDiscoveryUseCase,
    internal val pluginManager: PluginManager,
    internal val pluginRepositoryManager: PluginRepositoryManager,
    internal val gson: Gson,
    internal val profileManager: ProfileManager,
    internal val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
    internal val mdblistRatingsClient: MdblistRatingsClient,
    internal val httpEffectExecutor: HttpEffectExecutor,
    internal val playbackProgressScheduler: PlaybackProgressScheduler,
    internal val playbackSyncCoordinator: PlaybackSyncCoordinator,
    internal val providerAdapters: ProviderAdapters,
    internal val providerContinueWatchingRepository: ProviderContinueWatchingRepository,
    internal val thirdPartyProviderRepository: ThirdPartyProviderRepository,
    @param:Named("PluginScraperClient") internal val pluginScraperHttpClient: OkHttpClient
) : HeadlessPlatformEnvironment {

    internal val primeScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    internal val _streamProgressFlow = MutableSharedFlow<StreamProgressUpdate>(replay = 0, extraBufferCapacity = 32)
    val streamProgressFlow: SharedFlow<StreamProgressUpdate> = _streamProgressFlow

    internal val authEffectHandler = AndroidAuthEffectHandler(
        repository = repository,
        traktRepository = traktRepository,
        nuvioAccountImportCoordinator = nuvioAccountImportCoordinator,
        providerAdapters = providerAdapters,
        thirdPartyProviderRepository = thirdPartyProviderRepository,
        gson = gson
    )

    internal val calendarEffectHandler = AndroidCalendarEffectHandler(
        context = context,
        repository = repository,
        watchlistManager = watchlistManager,
        profileManager = profileManager,
        providerContinueWatchingRepository = providerContinueWatchingRepository,
        thirdPartyProviderRepository = thirdPartyProviderRepository,
        gson = gson
    )

    internal val offlineEffectHandler = AndroidOfflineEffectHandler(context, gson)
    internal val cloudStreamRuntime = AndroidCloudStreamRuntime(pluginManager)
    internal val trailerHttpClient = OkHttpClient.Builder().build()

    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion =
        withContext(Dispatchers.IO) {
            runCatching {
                syncWatchlistProfile(effect)
                dispatchEffect(effect)
            }.getOrElse { throwable ->
                Log.e("HeadlessEnv", "effect ${effect.type} failed", throwable)
                error(effect, throwable.message ?: throwable::class.java.simpleName)
            }
        }


    internal fun ok(effect: NativeHeadlessEffect, value: Any?): HeadlessEffectCompletion = HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = value)
    internal fun error(effect: NativeHeadlessEffect, code: String): HeadlessEffectCompletion = HeadlessEffectCompletion(effectId = effect.id, status = "error", error = mapOf("code" to code))
    internal fun Map<String, Any?>.profile(): UserProfile? = parseProfile(gson)
    internal fun Map<String, Any?>.remoteSources(): List<LibraryRemoteSource> {
        val raw = this["remoteSource"] ?: return emptyList()
        val values = raw as? List<*> ?: listOf(raw)
        return values.mapNotNull { value -> runCatching { gson.fromJson(gson.toJsonTree(value), LibraryRemoteSource::class.java) }.getOrNull() }
    }

    internal suspend fun buildPlaybackStreamRequestIds(
        type: String,
        id: String,
        language: String,
        profile: UserProfile?,
        timeoutMs: Long,
        prefetchedDetail: MetaDetail? = null
    ): List<String> {
        val detail = prefetchedDetail ?: if (StremioId.isTmdbLikeContentId(id) || type != "series") {
            withTimeoutOrNull(timeoutMs) {
                repository.getMetaDetail(
                    type = type,
                    id = StremioId.baseContentId(id),
                    language = language,
                    authKey = profile?.authKey.orEmpty(),
                    localAddons = profile?.safeLocalAddons.orEmpty(),
                    useConfiguredAddons = true
                )
            }
        } else null
        return FluxaCoreNative.playbackStreamRequestIds(type, id, detail?.id)
    }
}
