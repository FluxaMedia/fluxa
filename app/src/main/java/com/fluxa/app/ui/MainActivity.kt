@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.shared.FluxaDestination
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
import androidx.lifecycle.viewModelScope
import coil3.compose.AsyncImage
import androidx.compose.ui.layout.ContentScale
import com.fluxa.app.ui.catalog.*
import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.R
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : FragmentActivity() {

    @Inject lateinit var profileManager: ProfileManager
    @Inject lateinit var profilePickerSettingsStore: com.fluxa.app.data.local.ProfilePickerSettingsStore
    @Inject lateinit var pluginManager: PluginManager
    @Inject lateinit var pluginRepositoryManager: com.fluxa.app.plugins.PluginRepositoryManager
    @Inject lateinit var stremioRepository: StremioRepository
    @Inject lateinit var authService: StremioService
    @Inject lateinit var nuvioImportCoordinator: com.fluxa.app.data.repository.NuvioAccountImportCoordinator
    @Inject lateinit var nuvioSyncCoordinator: com.fluxa.app.data.repository.NuvioSyncCoordinator
    @Inject lateinit var watchlistStore: com.fluxa.app.data.local.WatchlistStore

    private val searchIntentFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
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
    }

    override fun onDestroy() {
        CommonActivity.activity = null
        com.fluxa.app.player.TorrentStreamManager.getInstance().shutdown()
        super.onDestroy()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        CommonActivity.activity = this
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (android.os.Build.VERSION.SDK_INT >= 33 &&
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
                    

                    var updateInfo by remember { mutableStateOf<UpdateManager.UpdateInfo?>(null) }
                    var downloadProgress by remember { mutableFloatStateOf(0f) }
                    var isDownloading by remember { mutableStateOf(false) }

                    val homeViewModel: HomeViewModel = androidx.hilt.navigation.compose.hiltViewModel()
                    val sharedDetailViewModel: com.fluxa.app.ui.catalog.DetailViewModel =
                        androidx.hilt.navigation.compose.hiltViewModel(key = "SharedMobileDetailViewModel")
                    val offlineDownloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }

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
                    val isDirectLoading by homeViewModel.isDirectLoading.collectAsState()
                    val traktContinueWatchingLastUpdatedAt by homeViewModel.traktContinueWatchingLastUpdatedAt.collectAsState()
                    val isNetworkAvailable by remember(context) {
                        context.observeNetworkAvailable()
                    }.collectAsState(initial = context.isNetworkAvailableNow())
                    var previousNetworkAvailable by remember { mutableStateOf<Boolean?>(null) }

                    LaunchedEffect(initialProfile) {
                        initialProfile?.let { profileManager.setLastActiveProfile(it) }
                    }

                    DisposableEffect(Unit) {
                        val listener = android.content.SharedPreferences.OnSharedPreferenceChangeListener { _, _ ->
                            profiles = profileManager.getProfiles()
                        }
                        profileManager.registerOnChangeListener(listener)
                        profiles = profileManager.getProfiles()
                        onDispose { profileManager.unregisterOnChangeListener(listener) }
                    }

                    LaunchedEffect(Unit) {
                        searchIntentFlow.collect { query ->
                            if (activeProfile == null) {
                                val profiles = profileManager.getProfiles()
                                activeProfile = profiles.firstOrNull()
                            }
                            if (activeProfile != null) {
                                navigateToDestination(FluxaDestination.Search, false)
                                homeViewModel.search(query)
                            }
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

                    val mainPlayer = remember(
                        activeProfile?.id,
                        activeProfile?.safeAudioDecoderMode,
                        activeProfile?.preferredAudioLanguage,
                        activeProfile?.playerBufferCacheMb,
                        activeProfile?.playerForwardBufferSeconds,
                        activeProfile?.playerBackBufferSeconds,
                        activeProfile?.tunneledPlayback,
                        activeProfile?.playerMinBufferSeconds,
                        activeProfile?.playerPlaybackBufferMs,
                        activeProfile?.playerRebufferBufferMs
                    ) {
                        MediaPlayerController.createExoPlayer(
                            context,
                            activeProfile?.safeAudioDecoderMode ?: "hw_prefer",
                            activeProfile?.preferredAudioLanguage?.takeUnless { it == "none" } ?: "",
                            activeProfile?.safePlayerBufferCacheMb ?: 100,
                            activeProfile?.safePlayerForwardBufferSeconds ?: 30,
                            activeProfile?.safePlayerBackBufferSeconds ?: 30,
                            activeProfile?.safeTunneledPlayback == true,
                            activeProfile?.safePlayerMinBufferSeconds ?: 8,
                            activeProfile?.safePlayerPlaybackBufferMs ?: 1500,
                            activeProfile?.safePlayerRebufferBufferMs ?: 2500
                        )
                    }
                    val previewPlayer = remember(activeProfile?.id, activeProfile?.safeAudioDecoderMode, activeProfile?.preferredAudioLanguage) {
                        val player = MediaPlayerController.createExoPlayer(
                            context,
                            activeProfile?.safeAudioDecoderMode ?: "hw_prefer",
                            activeProfile?.preferredAudioLanguage?.takeUnless { it == "none" } ?: ""
                        ).apply { volume = 0f }
                        MediaPlayerController(context, player)
                        player
                    }
                    val mediaSession = remember(mainPlayer) {
                        MediaSession.Builder(context, mainPlayer).build()
                    }
                    DisposableEffect(mediaSession) {
                        onDispose { mediaSession.release() }
                    }

                    val androidFluxaPlatformServices = remember(deviceType, homeViewModel, sharedDetailViewModel, profileManager, profilePickerSettingsStore) {
                        AndroidFluxaPlatformServices(
                            homeViewModel = homeViewModel,
                            detailViewModel = sharedDetailViewModel,
                            profileManager = profileManager,
                            profilePickerSettingsStore = profilePickerSettingsStore,
                            activeProfile = { activeProfile },
                            onActiveProfileChanged = { updated -> activeProfile = updated },
                            offlineDownloadManager = offlineDownloadManager,
                            watchlistStore = watchlistStore,
                            repository = stremioRepository,
                            pluginRepositoryManager = pluginRepositoryManager,
                            pluginManager = pluginManager,
                            authService = authService,
                            nuvioImportCoordinator = nuvioImportCoordinator,
                            appVersionLabel = "v${com.fluxa.app.BuildConfig.VERSION_NAME}"
                        )
                    }

                    PlayerLifecycleEffect(
                        isPlayerActive = playerRequest != null,
                        activeProfile = activeProfile,
                        mainPlayer = mainPlayer,
                        previewPlayer = previewPlayer,
                        homeViewModel = homeViewModel,
                        enterPictureInPicture = {
                            this@MainActivity.enterPictureInPictureMode(android.app.PictureInPictureParams.Builder().build())
                        }
                    )

                    LaunchedEffect(Unit) {
                        if (initialProfile == null) {
                            homeViewModel.loadInitialData(null)
                        }
                    }

                    AppUpdateCheckEffect(
                        automaticUpdatesEnabled = activeProfile?.safeAutomaticUpdates != false,
                        isDebugBuild = com.fluxa.app.BuildConfig.DEBUG,
                        onUpdateFound = { updateInfo = it }
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
                            profileManager = profileManager,
                            homeViewModel = homeViewModel,
                            mainPlayer = mainPlayer,
                            coroutineScope = coroutineScope,
                            offlineDownloadManager = offlineDownloadManager,
                            onShowTraktSheet = { showTraktSheet = true },
                            onShowSimklSheet = { showSimklSheet = true },
                            onTraktDeviceAuthChanged = { traktDeviceAuth = it },
                            onUpdateInfoChanged = { updateInfo = it },
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
