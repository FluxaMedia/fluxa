@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.shared.FluxaDestination
import com.fluxa.app.shared.feature.settings.SettingsUpdateCheckState
import com.fluxa.app.shared.feature.watchtogether.JvmWatchTogetherTransport
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherManager
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherCorrection
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.routes.AppRoutesHost
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.data.remote.StremioService

import android.content.pm.ActivityInfo
import android.os.Bundle
import android.util.Log
import com.lagradost.cloudstream3.CommonActivity
import androidx.fragment.app.FragmentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.Image
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import coil3.compose.AsyncImage
import androidx.compose.ui.layout.ContentScale
import com.fluxa.app.ui.catalog.*
import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.DiscordPresenceNative
import com.fluxa.app.R
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.flow.MutableStateFlow
import com.google.gson.JsonObject

import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : FragmentActivity() {

    @Inject lateinit var profileManager: ProfileManager
    @Inject lateinit var profilePickerSettingsStore: com.fluxa.app.data.local.ProfilePickerSettingsStore
    @Inject lateinit var pluginManager: PluginManager
    @Inject lateinit var pluginRepositoryManager: com.fluxa.app.plugins.PluginRepositoryManager
    @Inject lateinit var stremioRepository: StremioRepository
    @Inject lateinit var addonRepository: AddonRepository
    @Inject lateinit var platformSecureStore: com.fluxa.app.data.platform.PlatformSecureStore
    @Inject lateinit var authService: StremioService
    @Inject lateinit var nuvioService: com.fluxa.app.data.remote.NuvioService
    @Inject lateinit var fluxaSyncService: com.fluxa.app.data.remote.FluxaSyncService
    @Inject lateinit var nuvioImportCoordinator: com.fluxa.app.data.repository.NuvioAccountImportCoordinator
    @Inject lateinit var nuvioSyncCoordinator: com.fluxa.app.data.repository.NuvioSyncCoordinator
    @Inject lateinit var watchlistStore: com.fluxa.app.data.local.WatchlistStore
    @Inject lateinit var thirdPartyProviderRepository: com.fluxa.app.data.repository.library.ThirdPartyProviderRepository

    private val searchIntentFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val playbackDeepLinkFlow = MutableStateFlow<PlaybackDeepLink?>(null)
    private var playbackDeepLinkRevision = 0
    private val oauthRedirectHandler = OAuthRedirectHandler()

    override fun onNewIntent(intent: android.content.Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: android.content.Intent?) {
        if (intent == null) return
        val query = intent.getStringExtra(android.app.SearchManager.QUERY)
            ?: intent.getStringExtra("query")
            
        if (intent.action == android.content.Intent.ACTION_SEARCH ||
            intent.action == "com.google.android.gms.actions.SEARCH_ACTION" ||
            intent.action == "com.google.android.gms.actions.SEARCH_AND_PLAY" ||
            intent.action == "android.media.action.MEDIA_PLAY_FROM_SEARCH") {
            query?.let { searchIntentFlow.tryEmit(it) }
        }

        oauthRedirectHandler.handle(intent)

        val data = intent.data
        if (intent.action == android.content.Intent.ACTION_VIEW &&
            data?.scheme == "app" &&
            data.host == "play"
        ) {
            val id = data.getQueryParameter("id")?.takeIf(String::isNotBlank)
            val type = data.getQueryParameter("type")?.takeIf(String::isNotBlank)
            if (id != null && type != null) {
                playbackDeepLinkRevision += 1
                playbackDeepLinkFlow.value = PlaybackDeepLink(
                    revision = playbackDeepLinkRevision,
                    id = id,
                    type = type,
                    videoId = data.getQueryParameter("videoId"),
                    positionMs = data.getQueryParameter("positionMs")?.toLongOrNull()?.coerceAtLeast(0L) ?: 0L,
                    streamUrl = data.getQueryParameter("streamUrl"),
                    streamTitle = data.getQueryParameter("streamTitle"),
                )
            }
        }
    }

    override fun onDestroy() {
        CommonActivity.activity = null
        com.fluxa.app.player.TorrentStreamManager.getInstance().shutdown()
        super.onDestroy()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        CommonActivity.activity = this
        WatchTogetherManager.installTransportFactory { JvmWatchTogetherTransport() }
        WatchTogetherManager.installDriftPolicy { local, expected, hostPlaying, speedActive ->
            val correction = FluxaCoreNative.watchTogetherDriftCorrection(local, expected, hostPlaying, speedActive)
            when (correction["type"] as? String) {
                "seek" -> WatchTogetherCorrection.Seek((correction["positionMs"] as? Number)?.toLong() ?: expected)
                "speed" -> WatchTogetherCorrection.Speed((correction["value"] as? Number)?.toFloat() ?: 1f)
                "resetSpeed" -> WatchTogetherCorrection.ResetSpeed
                else -> WatchTogetherCorrection.None
            }
        }
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (!com.fluxa.app.BuildConfig.IS_TV && android.os.Build.VERSION.SDK_INT >= 33 &&
            androidx.core.content.ContextCompat.checkSelfPermission(this, android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED
        ) {
            androidx.core.app.ActivityCompat.requestPermissions(this, arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 1007)
        }
        
        handleIntent(intent)

        setContent {
            AppTheme {
                val context = LocalContext.current
                val deviceType = remember { if (com.fluxa.app.BuildConfig.IS_TV) DeviceType.TV else DeviceType.Mobile }
                
                CompositionLocalProvider(LocalDeviceType provides deviceType) {
                    var loadedInitialProfile by remember { mutableStateOf<UserProfile?>(null) }
                    var profilesReady by remember { mutableStateOf(false) }

                    LaunchedEffect(deviceType) {
                        val profile = withContext(Dispatchers.IO) {
                            initialProfileForDevice(profileManager, deviceType)
                        }
                        profile?.avatarUrl?.takeIf(String::isNotBlank)?.let { avatarUrl ->
                            val request = coil3.request.ImageRequest.Builder(this@MainActivity)
                                .data(com.fluxa.app.shared.image.sanitizeImageUrl(avatarUrl))
                                .memoryCacheKey("profile-avatar:$avatarUrl")
                                .diskCacheKey("profile-avatar:$avatarUrl")
                                .build()
                            coil3.SingletonImageLoader.get(this@MainActivity).execute(request)
                        }
                        loadedInitialProfile = profile
                        profilesReady = true
                    }

                    if (!profilesReady) {
                        Box(
                            modifier = Modifier.fillMaxSize().background(Color.Black),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator(color = Color.White)
                        }
                        return@CompositionLocalProvider
                    }

                    val initialProfile = loadedInitialProfile
                    val initialDestination = remember(initialProfile, deviceType) {
                        if (profileManager.getProfiles().isEmpty()) FluxaDestination.Auth
                        else initialDestinationForProfile(initialProfile)
                    }
                    var currentDestination by remember { mutableStateOf(initialDestination) }
                    var previousDestination by remember { mutableStateOf<FluxaDestination?>(null) }
                    var authStartOnNuvio by remember { mutableStateOf(false) }
                    var playerRequest by remember { mutableStateOf<PlayerLaunchRequest?>(null) }
                    var terminalDetailRequest by remember { mutableStateOf<com.fluxa.app.shared.feature.detail.DetailRequestUiModel?>(null) }
                    val navigateToDestination = { destination: FluxaDestination, clearStack: Boolean ->
                        previousDestination = if (clearStack || destination == currentDestination) null else currentDestination
                        currentDestination = destination
                    }

                    var activeProfile by remember { mutableStateOf<UserProfile?>(initialProfile) }
                    var profiles by remember { mutableStateOf(profileManager.getProfiles()) }
                    var traktDeviceAuth by remember { mutableStateOf<TraktDeviceAuthUiState?>(null) }
                    var showTraktSheet by remember { mutableStateOf(false) }
                    var isTraktSyncing by remember { mutableStateOf(false) }
                    var showSimklSheet by remember { mutableStateOf(false) }
                    val coroutineScope = rememberCoroutineScope()

                    LaunchedEffect(activeProfile?.id, activeProfile?.discordRichPresenceEnabled) {
                        DiscordPresenceNative.initialize(
                            activity = this@MainActivity,
                            enabled = activeProfile?.let { it.discordRichPresenceEnabled != false } == true,
                        )
                    }
                    

                    var updateInfo by remember { mutableStateOf<UpdateManager.UpdateInfo?>(null) }
                    var latestReleaseInfo by remember { mutableStateOf<UpdateManager.UpdateInfo?>(null) }
                    var releaseHistory by remember { mutableStateOf<List<UpdateManager.UpdateInfo>>(emptyList()) }
                    var updateCheckState by remember { mutableStateOf(SettingsUpdateCheckState.Idle) }
                    var downloadProgress by remember { mutableFloatStateOf(0f) }
                    var isDownloading by remember { mutableStateOf(false) }

                    val homeViewModel: HomeViewModel = hiltViewModel()
                    val sharedDetailViewModel: com.fluxa.app.ui.catalog.DetailViewModel =
                        hiltViewModel(key = "SharedMobileDetailViewModel")
                    val offlineDownloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }
                    val tvLauncherPublisher = remember(context) { AndroidTvLauncherPublisher(context) }
                    val tvLauncherCategories by homeViewModel.categories.collectAsStateWithLifecycle()
                    val tvLauncherBillboardItems by homeViewModel.billboardPool.collectAsStateWithLifecycle()
                    val tvLauncherWatchlist by homeViewModel.watchlist.collectAsStateWithLifecycle()

                    NuvioHealthSyncEffect(
                        profile = activeProfile,
                        homeViewModel = homeViewModel,
                        onProfileUpdated = { updated ->
                            activeProfile = updated
                            profileManager.saveProfile(updated)
                            profileManager.setLastActiveProfile(updated)
                            homeViewModel.applyUpdatedProfile(updated, refreshHomeSideEffects = true)
                        }
                    )
                    val isDirectLoading by homeViewModel.isDirectLoading.collectAsStateWithLifecycle()
                    val traktContinueWatchingLastUpdatedAt by homeViewModel.traktContinueWatchingLastUpdatedAt.collectAsStateWithLifecycle()
                    val isNetworkAvailable by remember(context) {
                        context.observeNetworkAvailable()
                    }.collectAsStateWithLifecycle(initialValue = context.isNetworkAvailableNow())
                    var previousNetworkAvailable by remember { mutableStateOf<Boolean?>(null) }

                    LaunchedEffect(initialProfile) {
                        initialProfile?.let { profileManager.setLastActiveProfile(it) }
                    }

                    LaunchedEffect(
                        tvLauncherCategories,
                        tvLauncherBillboardItems,
                        tvLauncherWatchlist,
                        activeProfile?.safeLanguage,
                    ) {
                        if (com.fluxa.app.BuildConfig.IS_TV) {
                            withContext(Dispatchers.IO) {
                                tvLauncherPublisher.publish(
                                    categories = tvLauncherCategories,
                                    billboardItems = tvLauncherBillboardItems,
                                    watchlist = tvLauncherWatchlist,
                                    language = activeProfile?.safeLanguage,
                                )
                            }
                        }
                    }

                    DisposableEffect(Unit) {
                        val listener = { profiles = profileManager.getProfiles() }
                        profileManager.addChangeListener(listener)
                        profiles = profileManager.getProfiles()
                        onDispose { profileManager.removeChangeListener(listener) }
                    }

                    LaunchedEffect(Unit) {
                        searchIntentFlow.collect { query ->
                            if (activeProfile == null) {
                                val profiles = profileManager.getProfiles()
                                activeProfile = profiles.firstOrNull()
                            }
                            if (activeProfile != null) {
                                navigateToDestination(FluxaDestination.Discover, false)
                                homeViewModel.search(query)
                            }
                        }
                    }

                    LaunchedEffect(Unit) {
                        playbackDeepLinkFlow.collect { link ->
                            if (link == null) return@collect
                            terminalDetailRequest = com.fluxa.app.shared.feature.detail.DetailRequestUiModel(
                                id = link.id,
                                type = link.type,
                                initialProgress = link.positionMs,
                                lastVideoId = link.videoId,
                                autoPlay = true,
                                lastStreamUrl = link.streamUrl,
                                lastStreamTitle = link.streamTitle,
                            )
                        }
                    }

                    OAuthRedirectEffect(
                        redirectHandler = oauthRedirectHandler,
                        context = context,
                        homeViewModel = homeViewModel,
                        profileManager = profileManager,
                        activeProfile = activeProfile,
                        onProfileUpdated = { activeProfile = it },
                        onTraktSyncingChanged = { isTraktSyncing = it }
                    )

                    TraktDeviceAuthDialog(
                        state = traktDeviceAuth,
                        lang = activeProfile?.safeLanguage,
                        onDismiss = { traktDeviceAuth = null }
                    )

                    val playerBufferTargets = remember(
                        activeProfile?.safePlayerBufferCacheMb,
                        activeProfile?.safePlayerForwardBufferSeconds,
                        activeProfile?.safePlayerBackBufferSeconds,
                        activeProfile?.safeMobileDataUsage
                    ) {
                        FluxaCoreNative.playerBufferTargets(
                            JsonObject().apply {
                                addProperty("cacheSizeMb", activeProfile?.safePlayerBufferCacheMb ?: 100)
                                addProperty("forwardBufferSeconds", activeProfile?.safePlayerForwardBufferSeconds ?: 30)
                                addProperty("backBufferSeconds", activeProfile?.safePlayerBackBufferSeconds ?: 30)
                                addProperty("mobileDataUsage", activeProfile?.safeMobileDataUsage ?: "medium")
                            }.toString()
                        )
                    }
                    val mainPlayer = remember(
                        playerRequest != null,
                        activeProfile?.id,
                        activeProfile?.safeAudioDecoderMode,
                        activeProfile?.safeAudioProcessingMode,
                        activeProfile?.preferredAudioLanguage,
                        playerBufferTargets.cacheSizeBytes,
                        playerBufferTargets.forwardBufferMs,
                        playerBufferTargets.backBufferMs,
                        activeProfile?.tunneledPlayback,
                        activeProfile?.playerMinBufferSeconds,
                        activeProfile?.playerPlaybackBufferMs,
                        activeProfile?.playerRebufferBufferMs
                    ) {
                        if (playerRequest == null) {
                            null
                        } else {
                            MediaPlayerController.createExoPlayer(
                                context,
                                activeProfile?.safeAudioDecoderMode ?: "hw_prefer",
                                activeProfile?.preferredAudioLanguage?.takeUnless { it == "none" } ?: "",
                                (playerBufferTargets.cacheSizeBytes / 1_000_000L).toInt(),
                                (playerBufferTargets.forwardBufferMs / 1_000L).toInt(),
                                (playerBufferTargets.backBufferMs / 1_000L).toInt(),
                                activeProfile?.safeTunneledPlayback == true,
                                activeProfile?.safePlayerMinBufferSeconds ?: 8,
                                activeProfile?.safePlayerPlaybackBufferMs ?: 1500,
                                activeProfile?.safePlayerRebufferBufferMs ?: 2500,
                                true,
                                activeProfile?.safeAudioProcessingMode ?: "reference"
                            )
                        }
                    }
                    DisposableEffect(mainPlayer) {
                        onDispose { mainPlayer?.let(MediaPlayerController::releaseExoPlayer) }
                    }
                    val androidFluxaPlatformServices = remember(deviceType, homeViewModel, sharedDetailViewModel, profileManager, profilePickerSettingsStore) {
                        AndroidFluxaPlatformServices(
                            context = context,
                            homeViewModel = homeViewModel,
                            detailViewModel = sharedDetailViewModel,
                            profileManager = profileManager,
                            profilePickerSettingsStore = profilePickerSettingsStore,
                            activeProfile = { activeProfile },
                            onActiveProfileChanged = { updated -> activeProfile = updated },
                            offlineDownloadManager = offlineDownloadManager,
                            watchlistStore = watchlistStore,
                            repository = stremioRepository,
                            addonRepository = addonRepository,
                            secureStore = platformSecureStore,
                            pluginRepositoryManager = pluginRepositoryManager,
                            pluginManager = pluginManager,
                            nuvioService = nuvioService,
                            fluxaSyncService = fluxaSyncService,
                            nuvioImportCoordinator = nuvioImportCoordinator,
                            nuvioSyncCoordinator = nuvioSyncCoordinator,
                            thirdPartyProviderRepository = thirdPartyProviderRepository,
                            appVersionLabel = "v${com.fluxa.app.BuildConfig.VERSION_NAME}",
                            deviceType = deviceType,
                        )
                    }

                    DisposableEffect(androidFluxaPlatformServices) {
                        onDispose { androidFluxaPlatformServices.close() }
                    }

                    PlayerLifecycleEffect(
                        isPlayerActive = playerRequest != null,
                        activeProfile = activeProfile,
                        mainPlayer = mainPlayer,
                        homeViewModel = homeViewModel,
                        enterPictureInPicture = {
                            this@MainActivity.enterPictureInPictureMode(android.app.PictureInPictureParams.Builder().build())
                        }
                    )

                    LaunchedEffect(Unit) {
                        if (initialProfile == null) {
                            homeViewModel.loadInitialData(null)
                        }
                        val releases = UpdateManager.fetchReleaseHistory()
                        releaseHistory = releases
                        latestReleaseInfo = releases.firstOrNull()
                    }

                    AppUpdateCheckEffect(
                        automaticUpdatesEnabled = activeProfile?.safeAutomaticUpdates != false,
                        isDebugBuild = com.fluxa.app.BuildConfig.DEBUG,
                        onUpdateFound = {
                            updateInfo = it
                            latestReleaseInfo = it
                            releaseHistory = listOf(it) + releaseHistory.filterNot { release -> release.versionName == it.versionName }
                            updateCheckState = SettingsUpdateCheckState.Available
                        }
                    )

                    LaunchedEffect(activeProfile?.id) {
                        activeProfile?.let { profile ->
                            homeViewModel.refreshTraktTokenIfNeeded(profile) { updated ->
                                activeProfile = updated
                                profileManager.saveProfile(updated)
                                profileManager.setLastActiveProfile(updated)
                            }
                            homeViewModel.loadInitialData(profile)
                            if (!profile.traktAccessToken.isNullOrBlank()) {
                                isTraktSyncing = true
                                homeViewModel.syncTraktIntegration(
                                    profile = profile,
                                    onProfileUpdated = { updated ->
                                        activeProfile = updated
                                        profileManager.saveProfile(updated)
                                        profileManager.setLastActiveProfile(updated)
                                    }
                                ) { isTraktSyncing = false }
                            }
                            if (!profile.simklAccessToken.isNullOrBlank() || !profile.anilistAccessToken.isNullOrBlank()) {
                                homeViewModel.loadLibraryItems(profile)
                            }
                        }
                    }

                    LaunchedEffect(isNetworkAvailable, activeProfile?.id) {
                        if (previousNetworkAvailable == false && isNetworkAvailable) {
                            activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
                        }
                        previousNetworkAvailable = isNetworkAvailable
                    }

                    LaunchedEffect(playerRequest != null) {
                        if (playerRequest != null) window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                        else window.clearFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                    }

                    DisposableEffect(playerRequest != null, deviceType) {
                        val controller = WindowInsetsControllerCompat(window, window.decorView)
                        if (deviceType == DeviceType.Mobile) {
                            WindowCompat.setDecorFitsSystemWindows(window, false)
                            if (playerRequest != null) {
                                controller.systemBarsBehavior =
                                    WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                                controller.hide(WindowInsetsCompat.Type.systemBars())
                            } else {
                                controller.show(WindowInsetsCompat.Type.systemBars())
                            }
                        } else {
                            WindowCompat.setDecorFitsSystemWindows(window, true)
                            controller.show(WindowInsetsCompat.Type.systemBars())
                        }
                        onDispose {
                            if (deviceType == DeviceType.Mobile) {
                                WindowCompat.setDecorFitsSystemWindows(window, false)
                            }
                            controller.show(WindowInsetsCompat.Type.systemBars())
                        }
                    }

                    var canPopSettings by remember { mutableStateOf(false) }
                    var settingsPopRequestId by remember { mutableStateOf(0) }
                    var hasOpenOverlay by remember { mutableStateOf(false) }
                    var overlayPopRequestId by remember { mutableStateOf(0) }

                    val navigateBackSafely = {
                        if (playerRequest != null) {
                            playerRequest = null
                        } else if (deviceType == DeviceType.Mobile && activeProfile == null && profileManager.getProfiles().isEmpty()) {
                            navigateToDestination(FluxaDestination.Auth, true)
                            authStartOnNuvio = false
                        } else if (activeProfile != null && previousDestination == null && currentDestination != FluxaDestination.Home) {
                            navigateToDestination(FluxaDestination.Home, true)
                        } else if (previousDestination != null) {
                            currentDestination = previousDestination!!
                            previousDestination = null
                        } else if (deviceType == DeviceType.TV) {
                            this@MainActivity.moveTaskToBack(true)
                        }
                    }

                    BackHandler(enabled = playerRequest == null) {
                        if (canPopSettings) {
                            settingsPopRequestId++
                        } else if (hasOpenOverlay) {
                            overlayPopRequestId++
                        } else {
                            navigateBackSafely()
                        }
                    }

                    Box(modifier = Modifier.fillMaxSize()) {
                        AppRoutesHost(
                            context = context,
                            currentDestination = currentDestination,
                            authStartOnNuvio = authStartOnNuvio,
                            playerRequest = playerRequest,
                            terminalDetailRequest = terminalDetailRequest,
                            deviceType = deviceType,
                            androidFluxaPlatformServices = androidFluxaPlatformServices,
                            activeProfile = activeProfile,
                            onActiveProfileChanged = { activeProfile = it },
                            settingsPopRequestId = settingsPopRequestId,
                            onSettingsCanPopChanged = { canPopSettings = it },
                            overlayPopRequestId = overlayPopRequestId,
                            onOverlayOpenChanged = { hasOpenOverlay = it },
                            onNavigateToDestination = { destination -> navigateToDestination(destination, false) },
                            onDestinationChanged = { destination -> currentDestination = destination },
                            onPlayerRequestChanged = { playerRequest = it },
                            onOpenTerminalDetail = { request -> terminalDetailRequest = request },
                            profileManager = profileManager,
                            homeViewModel = homeViewModel,
                            mainPlayer = mainPlayer,
                            coroutineScope = coroutineScope,
                            offlineDownloadManager = offlineDownloadManager,
                            onShowTraktSheet = { showTraktSheet = true },
                            onShowSimklSheet = { showSimklSheet = true },
                            onTraktDeviceAuthChanged = { traktDeviceAuth = it },
                            onUpdateInfoChanged = {
                                updateInfo = it
                                updateCheckState = if (it == null) SettingsUpdateCheckState.Idle else SettingsUpdateCheckState.Available
                            },
                            updateInfo = updateInfo,
                            latestReleaseInfo = latestReleaseInfo,
                            onLatestReleaseInfoChanged = { latestReleaseInfo = it },
                            releaseHistory = releaseHistory,
                            onReleaseHistoryChanged = { releaseHistory = it },
                            updateCheckState = updateCheckState,
                            onUpdateCheckStateChanged = { updateCheckState = it },
                            navigateBackSafely = navigateBackSafely
                        )

                        AppChromeOverlays(
                            context = context,
                            applicationContext = applicationContext,
                            deviceType = deviceType,
                            activeProfile = activeProfile,
                            onActiveProfileChanged = { activeProfile = it },
                            profileManager = profileManager,
                            homeViewModel = homeViewModel,
                            updateInfo = updateInfo,
                            isDownloading = isDownloading,
                            downloadProgress = downloadProgress,
                            isDirectLoading = isDirectLoading,
                            showTraktSheet = showTraktSheet,
                            isTraktSyncing = isTraktSyncing,
                            traktContinueWatchingLastUpdatedAt = traktContinueWatchingLastUpdatedAt,
                            showSimklSheet = showSimklSheet,
                            onUpdateInfoChanged = { updateInfo = it },
                            onDownloadingChanged = { isDownloading = it },
                            onDownloadProgressChanged = { downloadProgress = it },
                            onShowTraktSheetChanged = { showTraktSheet = it },
                            onTraktSyncingChanged = { isTraktSyncing = it },
                            onShowSimklSheetChanged = { showSimklSheet = it }
                        )
                    }
                }
            }
        }
    }
}

private data class PlaybackDeepLink(
    val revision: Int,
    val id: String,
    val type: String,
    val videoId: String?,
    val positionMs: Long,
    val streamUrl: String?,
    val streamTitle: String?,
)
