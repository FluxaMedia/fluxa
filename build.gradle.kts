import org.gradle.api.GradleException
import org.gradle.api.tasks.Exec

// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id("com.android.application") version "8.13.2" apply false
    id("org.jetbrains.kotlin.android") version "2.3.0" apply false
    id("com.google.devtools.ksp") version "2.3.0" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.3.0" apply false
    id("com.google.dagger.hilt.android") version "2.58" apply false
}

val maxKotlinFileLines = 1200
val rustCoreProjectDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core").canonicalFile
val rustCoreDelegateFiles = mapOf(
    "app/src/main/java/com/fluxa/app/domain/discovery/StremioAddonUrls.kt" to listOf(
        "FluxaCoreNative.normalizeManifestUrl",
        "FluxaCoreNative.identity",
        "FluxaCoreNative.manifestCandidates",
        "FluxaCoreNative.baseUrl",
        "FluxaCoreNative.preferHttpsAssetUrl"
    ),
    "app/src/main/java/com/fluxa/app/domain/discovery/StremioAddonProtocol.kt" to listOf(
        "FluxaCoreNative.supportsResource"
    ),
    "app/src/main/java/com/fluxa/app/core/StremioId.kt" to listOf(
        "FluxaCoreNative.parseEpisodeLocator",
        "FluxaCoreNative.streamRequestIds"
    ),
    "app/src/main/java/com/fluxa/app/data/remote/Stream.kt" to listOf(
        "FluxaCoreNative.streamPlaybackInfo"
    ),
    "app/src/main/java/com/fluxa/app/player/TorrentStreamManager.kt" to listOf(
        "TorrentCorePolicy.plan",
        "TorrentCorePolicy.statusInfo"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/PlayerScreenHelpers.kt" to listOf(
        "FluxaCoreNative.streamMatchesEpisode",
        "FluxaCoreNative.streamPlaybackInfo",
        "FluxaCoreNative.isTorrentPlaybackUrl"
    ),
    "app/src/main/java/com/fluxa/app/data/repository/StremioAddonManifestClient.kt" to listOf(
        "FluxaCoreNative.buildResourceUrl",
        "FluxaCoreNative.manifestFetchPlan",
        "FluxaCoreNative.fetchAddonResource",
        "FluxaCoreNative.parseManifestJson",
        "FluxaCoreNative.resolveManifestAssets",
        "FluxaCoreNative.mergeLiveManifest"
    ),
    "app/src/main/java/com/fluxa/app/data/repository/StremioAddonResourceClient.kt" to listOf(
        "FluxaCoreNative.fetchAddonResource",
        "FluxaCoreNative.parseAddonResourceResult",
        "FluxaCoreNative.parseExtraArgs"
    ),
    "app/src/main/java/com/fluxa/app/player/TorrServerEngine.kt" to listOf(
        "FluxaCoreNative.startTorrentServer",
        "FluxaCoreNative.stopTorrentServer"
    ),
    "app/src/main/java/com/fluxa/app/player/TorrentCorePolicy.kt" to listOf(
        "FluxaCoreNative.torrentRuntimeInfo",
        "FluxaCoreNative.torrentStatusInfo"
    ),
    "app/src/main/java/com/fluxa/app/player/MediaPlayerController.kt" to listOf(
        "FluxaCoreNative.rewriteDolbyVisionProfile7Codecs"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/StreamSourceSelectionPolicy.kt" to listOf(
        "FluxaCoreNative.selectStreamIndex"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/ContinueWatchingListMerger.kt" to listOf(
        "FluxaCoreNative.mergeContinueWatchingDuplicates"
    ),
    "app/src/main/java/com/fluxa/app/domain/discovery/DiscoverCatalogContentLoader.kt" to listOf(
        "FluxaCoreNative.filterDiscoverResults",
        "FluxaCoreNative.discoverCatalogCacheKey",
        "FluxaCoreNative.providerSearchTerms"
    ),
    "app/src/main/java/com/fluxa/app/domain/discovery/StreamDiscovery.kt" to listOf(
        "FluxaCoreNative.streamDiscoveryExecutionPolicy"
    ),
    "app/src/main/java/com/fluxa/app/domain/discovery/MetadataFeeds.kt" to listOf(
        "FluxaCoreNative.discoverCatalogLabel",
        "FluxaCoreNative.normalizeContentType",
        "FluxaCoreNative.stableFeedPart",
        "FluxaCoreNative.metadataFeedHomeTitle",
        "FluxaCoreNative.effectiveMetadataFeedSelection",
        "FluxaCoreNative.toggleMetadataFeed",
        "FluxaCoreNative.setMetadataFeedGroupEnabled",
        "FluxaCoreNative.orderedMetadataFeedKeys",
        "FluxaCoreNative.moveMetadataFeedOrder"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/ContentIdentity.kt" to listOf(
        "FluxaCoreNative.contentTraktKey",
        "FluxaCoreNative.contentBillboardKey",
        "FluxaCoreNative.contentMergeKeys",
        "FluxaCoreNative.contentWatchedKeys",
        "FluxaCoreNative.normalizedBillboardTitle"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/PlayerPlaybackRuntimeEffects.kt" to listOf(
        "viewModel.resolvePlayerPlayback"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/PlayerLoadingEffects.kt" to listOf(
        "viewModel.loadPlayerStreams"
    ),
    "app/src/main/java/com/fluxa/app/core/rust/FluxaHeadlessEffectRunner.kt" to listOf(
        "FluxaHeadlessEngine",
        "HeadlessPlatformEnvironment"
    ),
    "app/src/main/java/com/fluxa/app/core/rust/FluxaCoreNative.kt" to listOf(
        "NativeCoreCapabilitySet",
        "coreCapabilitiesJsonNative"
    ),
    "app/src/main/java/com/fluxa/app/core/rust/FluxaCoreUniFfi.kt" to listOf(
        "com.fluxa.core.uniffi",
        "FluxaHeadlessEngine"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/PlayerScrobbleCoordinator.kt" to listOf(
        "FluxaCoreNative.playerProgressPercent",
        "FluxaCoreNative.playerShouldSendScrobbleStart",
        "FluxaCoreNative.playerShouldMarkScrobbleStopped",
        "FluxaCoreNative.playerShouldQueueScrobblePause",
        "FluxaCoreNative.playerShouldEnqueueDurableScrobble",
        "FluxaCoreNative.playerShouldSavePeriodicProgress",
        "FluxaCoreNative.playerShouldSaveOnDispose"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/TrackSelectionState.kt" to listOf(
        "FluxaCoreNative.playerTrackState",
        "FluxaCoreNative.subtitleLanguageMatches",
        "preferredSubtitleIndex"
    ),
    "app/src/main/java/com/fluxa/app/data/repository/TraktIntegration.kt" to listOf(
        "FluxaCoreNative.traktHasClient",
        "FluxaCoreNative.traktBearer",
        "FluxaCoreNative.traktScrobbleUrl",
        "FluxaCoreNative.traktPlaybackUrl",
        "FluxaCoreNative.traktTokenExpiresAt",
        "FluxaCoreNative.traktContentIdFrom",
        "FluxaCoreNative.traktIdsFromContentId",
        "FluxaCoreNative.traktEpisodeLocator",
        "FluxaCoreNative.traktShowIdFromEpisodeId",
        "FluxaCoreNative.traktScrobbleMediaId",
        "FluxaCoreNative.traktHistoryRequest"
    ),
    "app/src/main/java/com/fluxa/app/data/repository/StremioRepository.kt" to listOf(
        "FluxaCoreNative.libraryContinueWatchingItems",
        "FluxaCoreNative.watchedVideoIds",
        "FluxaCoreNative.playbackProgressItem",
        "FluxaCoreNative.clearPlaybackProgressItem",
        "FluxaCoreNative.watchedStateItems",
        "FluxaCoreNative.traktHistoryRequest"
    ),
    "app/src/main/java/com/fluxa/app/data/local/ProfileManager.kt" to listOf(
        "FluxaCoreNative.sanitizeProfile",
        "FluxaCoreNative.profileLocalAddonsKey",
        "FluxaCoreNative.safePlayerBufferCacheMb",
        "FluxaCoreNative.safeDolbyVisionFallbackMode",
        "FluxaCoreNative.safeStreamSourceSelectionMode"
    ),
    "app/src/main/java/com/fluxa/app/plugins/PluginManager.kt" to listOf(
        "FluxaCoreNative.normalizePluginRepositoryUrl",
        "FluxaCoreNative.pluginIsSecureRemoteUrl",
        "FluxaCoreNative.pluginSameRepositoryUrl"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/DetailViewModel.kt" to listOf(
        "FluxaHeadlessRuntimeFactory",
        "FluxaAndroidHeadlessEnvironment",
        "detailLoadRequested",
        "detailSelectedAddonChanged",
        "markWatchedRequested",
        "toggleWatchlistRequested"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/HomeViewModel.kt" to listOf(
        "FluxaHeadlessRuntimeFactory",
        "FluxaAndroidHeadlessEnvironment",
        "homeLoadRequested"
    )
)

tasks.register("checkKotlinFileSize") {
    group = "verification"
    description = "Fails when Kotlin source files exceed the local maintainability budget."

    doLast {
        val oversizedFiles = fileTree(rootDir) {
            include("app/src/main/java/**/*.kt")
            exclude("**/build/**")
        }.files.mapNotNull { file ->
            val lines = file.readLines().size
            if (lines > maxKotlinFileLines) "${file.relativeTo(rootDir)}: $lines" else null
        }

        if (oversizedFiles.isNotEmpty()) {
            throw GradleException(
                "Kotlin files exceed $maxKotlinFileLines lines:\n${oversizedFiles.joinToString("\n")}"
            )
        }
    }
}

tasks.register("checkRustCoreBoundary") {
    group = "verification"
    description = "Fails when platform-independent core behavior stops delegating to ../fluxa-core."

    doLast {
        val missingDelegates = rustCoreDelegateFiles.flatMap { (relativePath, requiredCalls) ->
            val file = rootProject.file(relativePath)
            if (!file.exists()) {
                return@flatMap listOf("$relativePath is missing")
            }
            val text = file.readText()
            requiredCalls
                .filterNot { call -> text.contains(call) }
                .map { call -> "$relativePath must delegate to $call" }
        }

        val urlFacade = rootProject.file("app/src/main/java/com/fluxa/app/domain/discovery/StremioAddonUrls.kt")
        val duplicatedUrlLogic = if (urlFacade.exists()) {
            val text = urlFacade.readText()
            listOf("http://", "https://", "stremio://", "manifest.json", "Regex(")
                .filter { token -> text.contains(token) }
                .map { token -> "${urlFacade.relativeTo(rootDir)} must not reimplement URL rules containing `$token`" }
        } else {
            emptyList()
        }

        val viewModelBackendRules = mapOf(
            "app/src/main/java/com/fluxa/app/ui/catalog/HomeViewModel.kt" to listOf(
                "repository.get",
                "addonRepository.get",
                "watchlistManager.get",
                "watchlistManager.save",
                "watchlistManager.toggle",
                "traktRepository.get",
                "StreamDiscoveryUseCase",
                "HomePlaybackStreamCoordinator",
                "HomeAddonCoordinator",
                "HomeUserContentActions",
                "HomeTraktCoordinator",
                "streamDiscovery.",
                "pluginManager."
            ),
            "app/src/main/java/com/fluxa/app/ui/catalog/DetailViewModel.kt" to listOf(
                "com.fluxa.app.data.repository",
                "repository.",
                "watchlistManager.",
                "streamDiscovery.",
                "pluginManager.",
                "TorrentStreamManager"
            )
        )
        val viewModelBackendViolations = viewModelBackendRules.flatMap { (relativePath, bannedTokens) ->
            val file = rootProject.file(relativePath)
            if (!file.exists()) {
                return@flatMap emptyList()
            }
            val text = file.readText()
            bannedTokens
                .filter { token -> text.contains(token) }
                .map { token -> "$relativePath must route backend behavior through Rust headless actions, found `$token`" }
        }

        val failures = missingDelegates + duplicatedUrlLogic + viewModelBackendViolations
        if (failures.isNotEmpty()) {
            throw GradleException(
                "Rust core boundary violations:\n${failures.joinToString("\n")}"
            )
        }
    }
}

tasks.register("qualityCheck") {
    group = "verification"
    description = "Runs the default local quality gate for Fluxa."
    dependsOn(
        "checkKotlinFileSize",
        "checkRustCoreBoundary",
        ":app:testMobileDebugUnitTest",
        ":app:assembleMobileDebug",
        ":app:assembleTvDebug"
    )
}

tasks.register<Exec>("buildFluxaCoreHost") {
    group = "build"
    description = "Builds the Fluxa Rust core for the host toolchain."
    workingDir = rustCoreProjectDir
    commandLine("cargo", "build")
}

tasks.register("buildFluxaCoreAndroid") {
    group = "build"
    description = "Builds the Fluxa Rust core for Android JNI ABIs."
    dependsOn(":app:buildFluxaCore")
}
