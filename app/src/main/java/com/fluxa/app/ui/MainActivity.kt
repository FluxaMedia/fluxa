@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.routes.DetailRoute
import com.fluxa.app.ui.routes.AppRoutesHost
import com.fluxa.app.ui.routes.HomeRoute
import com.fluxa.app.ui.routes.PlayerRoute
import com.fluxa.app.ui.routes.SettingsRoute
import com.fluxa.app.ui.routes.SourcesRoute
import com.fluxa.app.ui.routes.detailScreen
import com.fluxa.app.ui.routes.sourcesScreen
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
    @Inject lateinit var pluginManager: PluginManager
    @Inject lateinit var stremioRepository: StremioRepository
    @Inject lateinit var authService: StremioService
    @Inject lateinit var nuvioImportCoordinator: com.fluxa.app.data.repository.NuvioAccountImportCoordinator

    private val searchIntentFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val traktAuthFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val malAuthFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val simklAuthFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private val anilistAuthFlow = kotlinx.coroutines.flow.MutableSharedFlow<String>(extraBufferCapacity = 1)
    private var pendingMalCodeVerifier: String? = null
    private val oauthPrefs by lazy { getSharedPreferences("fluxa_oauth", MODE_PRIVATE) }

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

        if (intent.action == android.content.Intent.ACTION_VIEW) {
            val data = intent.data ?: return
            val isOAuthRedirect = data.scheme == "fluxa" && data.host == "oauth"
            if (isOAuthRedirect && data.lastPathSegment == "trakt") {
                data.getQueryParameter("code")?.let { traktAuthFlow.tryEmit(it) }
            }
            if (isOAuthRedirect && data.lastPathSegment == "mal") {
                data.getQueryParameter("code")?.let { malAuthFlow.tryEmit(it) }
            }
            if (isOAuthRedirect && data.lastPathSegment == "simkl") {
                data.getQueryParameter("code")?.let { simklAuthFlow.tryEmit(it) }
            }
            if (isOAuthRedirect && data.lastPathSegment == "anilist") {
                data.getQueryParameter("code")?.let { anilistAuthFlow.tryEmit(it) }
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
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (android.os.Build.VERSION.SDK_INT >= 33 &&
            androidx.core.content.ContextCompat.checkSelfPermission(this, android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED
        ) {
            androidx.core.app.ActivityCompat.requestPermissions(this, arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 1007)
        }
        
        handleIntent(intent)

        AppContainer.initialize(profileManager, pluginManager, stremioRepository, authService, nuvioImportCoordinator)

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
                    val initialScreen = remember(initialProfile, deviceType) {
                        if (profileManager.getProfiles().isEmpty() && deviceType == DeviceType.Mobile) Screen.Login()
                        else if (profileManager.getProfiles().isEmpty()) Screen.Welcome
                        else initialScreenForProfile(initialProfile)
                    }
                    val navigator = rememberAppNavigator(initialScreen)

                    var activeProfile by remember { mutableStateOf<UserProfile?>(initialProfile) }
                    var profiles by remember { mutableStateOf(profileManager.getProfiles()) }
                    var traktDeviceAuth by remember { mutableStateOf<TraktDeviceAuthUiState?>(null) }
                    var showTraktSheet by remember { mutableStateOf(false) }
                    var isTraktSyncing by remember { mutableStateOf(false) }
                    var showMalSheet by remember { mutableStateOf(false) }
                    var showSimklSheet by remember { mutableStateOf(false) }
                    val coroutineScope = rememberCoroutineScope()
                    

                    val currentScreen = navigator.currentScreen
                    var updateInfo by remember { mutableStateOf<UpdateManager.UpdateInfo?>(null) }
                    var downloadProgress by remember { mutableFloatStateOf(0f) }
                    var isDownloading by remember { mutableStateOf(false) }

                    val homeViewModel: HomeViewModel = androidx.hilt.navigation.compose.hiltViewModel()
                    val sharedDetailViewModel: com.fluxa.app.ui.catalog.DetailViewModel =
                        androidx.hilt.navigation.compose.hiltViewModel(key = "SharedMobileDetailViewModel")
                    val offlineDownloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }
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
                                navigator.navigateTo(Screen.Search)
                                homeViewModel.search(query)
                            }
                        }
                    }

                    LaunchedEffect(Unit) {
                        traktAuthFlow.collect { code ->
                            homeViewModel.exchangeTraktCode(code, onProfileUpdated = { updated ->
                                activeProfile = updated
                                profileManager.saveProfile(updated)
                            }) { success ->
                                android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, if (success) "toast.trakt_connected" else "toast.trakt_connect_failed"), android.widget.Toast.LENGTH_SHORT).show()
                                activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
                                if (success) {
                                    activeProfile?.let { profile ->
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
                        }
                    }

                    LaunchedEffect(Unit) {
                        malAuthFlow.collect { code ->
                            val verifier = pendingMalCodeVerifier ?: oauthPrefs.getString("mal_code_verifier", null)
                            if (verifier.isNullOrBlank()) {
                                android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.mal_verifier_missing"), android.widget.Toast.LENGTH_SHORT).show()
                            } else {
                                homeViewModel.exchangeMalCode(code, verifier, onProfileUpdated = { updated ->
                                    activeProfile = updated
                                    profileManager.saveProfile(updated)
                                }) { success ->
                                    if (success) {
                                        pendingMalCodeVerifier = null
                                        oauthPrefs.edit().remove("mal_code_verifier").apply()
                                    }
                                    android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, if (success) "toast.mal_connected" else "toast.mal_connect_failed"), android.widget.Toast.LENGTH_SHORT).show()
                                    activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
                                }
                            }
                        }
                    }

                    LaunchedEffect(Unit) {
                        simklAuthFlow.collect { code ->
                            homeViewModel.exchangeSimklCode(code, onProfileUpdated = { updated ->
                                activeProfile = updated
                                profileManager.saveProfile(updated)
                            }) { success ->
                                android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, if (success) "toast.simkl_connected" else "toast.simkl_connect_failed"), android.widget.Toast.LENGTH_SHORT).show()
                                activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
                            }
                        }
                    }

                    LaunchedEffect(Unit) {
                        anilistAuthFlow.collect { code ->
                            homeViewModel.exchangeAnilistCode(code, onProfileUpdated = { updated ->
                                activeProfile = updated
                                profileManager.saveProfile(updated)
                            }) { success ->
                                android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, if (success) "toast.anilist_connected" else "toast.anilist_connect_failed"), android.widget.Toast.LENGTH_SHORT).show()
                                activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
                            }
                        }
                    }

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

                    val androidFluxaPlatformServices = remember(deviceType, mainPlayer, homeViewModel, sharedDetailViewModel, profileManager) {
                        if (deviceType == DeviceType.Mobile) {
                            AndroidFluxaPlatformServices(
                                homeViewModel = homeViewModel,
                                detailViewModel = sharedDetailViewModel,
                                profileManager = profileManager,
                                activeProfile = { activeProfile },
                                onActiveProfileChanged = { updated -> activeProfile = updated },
                                player = mainPlayer,
                                playerContent = { null },
                                offlineDownloadManager = offlineDownloadManager,
                                appVersionLabel = "v${com.fluxa.app.BuildConfig.VERSION_NAME}"
                            )
                        } else {
                            null
                        }
                    }

                    PlayerLifecycleEffect(
                        currentScreen = currentScreen,
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

                    LaunchedEffect(activeProfile?.safeAutomaticUpdates) {
                        if (activeProfile?.safeAutomaticUpdates == false) return@LaunchedEffect
                        while (isActive) {
                            val update = UpdateManager.checkUpdate()
                            if (update != null) {
                                updateInfo = update
                                break
                            }
                            delay(if (com.fluxa.app.BuildConfig.DEBUG) 120000 else 21600000)
                        }
                    }

                    LaunchedEffect(activeProfile?.id) {
                        activeProfile?.let { profile ->
                            homeViewModel.refreshTraktTokenIfNeeded(profile) { updated ->
                                activeProfile = updated
                                profileManager.saveProfile(updated)
                                profileManager.setLastActiveProfile(updated)
                            }
                            homeViewModel.refreshMalTokenIfNeeded(profile) { updated ->
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

                    LaunchedEffect(currentScreen) {
                        if (currentScreen is Screen.Player) window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                        else window.clearFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                    }

                    DisposableEffect(currentScreen, deviceType) {
                        val controller = WindowInsetsControllerCompat(window, window.decorView)
                        if (deviceType == DeviceType.Mobile) {
                            WindowCompat.setDecorFitsSystemWindows(window, false)
                            if (currentScreen is Screen.Player) {
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

                    val navigateBackSafely = {
                        if (deviceType == DeviceType.Mobile && activeProfile == null && profileManager.getProfiles().isEmpty()) {
                            navigator.navigateTo(Screen.Login(), true)
                        } else if (deviceType == DeviceType.Mobile && activeProfile != null && !navigator.canNavigateBack() && currentScreen !is Screen.Home) {
                            navigator.navigateTo(Screen.Home, true)
                        } else {
                            navigator.navigateBack()
                        }
                    }

                    BackHandler(enabled = currentScreen !is Screen.Player) { navigateBackSafely() }

                    Box(modifier = Modifier.fillMaxSize()) {
                        AppRoutesHost(
                            context = context,
                            currentScreen = currentScreen,
                            deviceType = deviceType,
                            androidFluxaPlatformServices = androidFluxaPlatformServices,
                            sharedDetailViewModel = sharedDetailViewModel,
                            activeProfile = activeProfile,
                            onActiveProfileChanged = { activeProfile = it },
                            navigator = navigator,
                            profileManager = profileManager,
                            homeViewModel = homeViewModel,
                            previewPlayer = previewPlayer,
                            mainPlayer = mainPlayer,
                            coroutineScope = coroutineScope,
                            offlineDownloadManager = offlineDownloadManager,
                            oauthPrefs = oauthPrefs,
                            onShowTraktSheet = { showTraktSheet = true },
                            onShowMalSheet = { showMalSheet = true },
                            onShowSimklSheet = { showSimklSheet = true },
                            onTraktDeviceAuthChanged = { traktDeviceAuth = it },
                            onPendingMalCodeVerifierChanged = { pendingMalCodeVerifier = it },
                            onUpdateInfoChanged = { updateInfo = it },
                            navigateBackSafely = navigateBackSafely
                        )

                        AppChromeOverlays(
                            context = context,
                            applicationContext = applicationContext,
                            currentScreen = currentScreen,
                            deviceType = deviceType,
                            activeProfile = activeProfile,
                            profiles = profiles,
                            onActiveProfileChanged = { activeProfile = it },
                            onQuickProfileSelected = { profile ->
                                activeProfile = profile
                                profileManager.setLastActiveProfile(profile)
                            },
                            profileManager = profileManager,
                            navigator = navigator,
                            homeViewModel = homeViewModel,
                            updateInfo = updateInfo,
                            isDownloading = isDownloading,
                            downloadProgress = downloadProgress,
                            isDirectLoading = isDirectLoading,
                            showTraktSheet = showTraktSheet,
                            isTraktSyncing = isTraktSyncing,
                            traktContinueWatchingLastUpdatedAt = traktContinueWatchingLastUpdatedAt,
                            showMalSheet = showMalSheet,
                            showSimklSheet = showSimklSheet,
                            onUpdateInfoChanged = { updateInfo = it },
                            onDownloadingChanged = { isDownloading = it },
                            onDownloadProgressChanged = { downloadProgress = it },
                            onShowTraktSheetChanged = { showTraktSheet = it },
                            onTraktSyncingChanged = { isTraktSyncing = it },
                            onShowMalSheetChanged = { showMalSheet = it },
                            onShowSimklSheetChanged = { showSimklSheet = it }
                        )
                    }
                }
            }
        }
    }
}
