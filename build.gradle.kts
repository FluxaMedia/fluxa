import org.gradle.api.GradleException
import org.gradle.api.tasks.Exec

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.compose.multiplatform) apply false
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.hilt) apply false
}

val maxKotlinFileLines = 1200
val rustCoreProjectDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core").canonicalFile
val rustHostLibraryName = when {
    org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "libfluxa_core.dylib"
    org.gradle.internal.os.OperatingSystem.current().isWindows -> "fluxa_core.dll"
    else -> "libfluxa_core.so"
}
val rustStreamingHostLibraryName = when {
    org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "libfluxa_streaming_engine.dylib"
    org.gradle.internal.os.OperatingSystem.current().isWindows -> "fluxa_streaming_engine.dll"
    else -> "libfluxa_streaming_engine.so"
}
val rustCoreDelegateFiles = mapOf(
    "data/src/main/java/com/fluxa/app/domain/discovery/StremioAddonUrls.kt" to listOf(
        "FluxaCoreNative.normalizeManifestUrl",
        "FluxaCoreNative.identity",
        "FluxaCoreNative.manifestCandidates",
        "FluxaCoreNative.baseUrl",
        "FluxaCoreNative.preferHttpsAssetUrl"
    ),
    "data/src/main/java/com/fluxa/app/domain/discovery/StremioAddonProtocol.kt" to listOf(
        "FluxaCoreNative.supportsResource"
    ),
    "app/src/main/java/com/fluxa/app/core/StremioId.kt" to listOf(
        "FluxaCoreNative.parseEpisodeLocator",
        "FluxaCoreNative.streamRequestIds"
    ),
    "data/src/main/java/com/fluxa/app/data/remote/Stream.kt" to listOf(
        "FluxaCoreNative.streamPlaybackInfo"
    ),
    "player/src/main/java/com/fluxa/app/player/TorrentStreamManager.kt" to listOf(
        "TorrentCorePolicy.plan",
        "TorrentCorePolicy.statusInfo"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/shared/player/PlayerScreenHelpers.kt" to listOf(
        "FluxaCoreNative.streamMatchesEpisode",
        "FluxaCoreNative.streamPlaybackInfo",
        "FluxaCoreNative.isTorrentPlaybackUrl"
    ),
    "data/src/main/java/com/fluxa/app/data/repository/StremioAddonManifestClient.kt" to listOf(
        "FluxaCoreNative.buildResourceUrl",
        "FluxaCoreNative.manifestFetchPlan",
        "FluxaCoreNative.fetchAddonResource",
        "FluxaCoreNative.parseManifestJson",
        "FluxaCoreNative.resolveManifestAssets",
        "FluxaCoreNative.mergeLiveManifest"
    ),
    "data/src/main/java/com/fluxa/app/data/repository/StremioAddonResourceClient.kt" to listOf(
        "FluxaCoreNative.fetchAddonResource",
        "FluxaCoreNative.parseAddonResourceResult",
        "FluxaCoreNative.parseExtraArgs"
    ),
    "player/src/main/java/com/fluxa/app/player/TorrentServerEngine.kt" to listOf(
        "FluxaCoreNative.startTorrentServer",
        "FluxaCoreNative.stopTorrentServer"
    ),
    "player/src/main/java/com/fluxa/app/player/TorrentCorePolicy.kt" to listOf(
        "FluxaCoreNative.torrentRuntimeInfo",
        "FluxaCoreNative.torrentStatusInfo"
    ),
    "player/src/main/java/com/fluxa/app/player/MediaPlayerController.kt" to listOf(
        "FluxaCoreNative.rewriteDolbyVisionProfile7Codecs"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/StreamSourceSelectionPolicy.kt" to listOf(
        "FluxaCoreNative.selectStreamIndex"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/ContinueWatchingListMerger.kt" to listOf(
        "FluxaCoreNative.mergeContinueWatchingDuplicates"
    ),
    "data/src/main/java/com/fluxa/app/domain/discovery/DiscoverCatalogContentLoader.kt" to listOf(
        "FluxaCoreNative.filterDiscoverResults",
        "FluxaCoreNative.discoverCatalogCacheKey",
        "FluxaCoreNative.providerSearchTerms"
    ),
    "data/src/main/java/com/fluxa/app/domain/discovery/StreamDiscovery.kt" to listOf(
        "FluxaCoreNative.streamDiscoveryExecutionPolicy"
    ),
    "data/src/main/java/com/fluxa/app/domain/discovery/MetadataFeeds.kt" to listOf(
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
    "data/src/main/java/com/fluxa/app/data/repository/TraktIntegration.kt" to listOf(
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
    "data/src/main/java/com/fluxa/app/data/repository/StremioRepository.kt" to listOf(
        "FluxaCoreNative.libraryContinueWatchingItems",
        "FluxaCoreNative.watchedVideoIds",
        "FluxaCoreNative.playbackProgressItem",
        "FluxaCoreNative.clearPlaybackProgressItem",
        "FluxaCoreNative.watchedStateItems",
        "FluxaCoreNative.traktHistoryRequest"
    ),
    "data/src/main/java/com/fluxa/app/data/local/ProfileManager.kt" to listOf(
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

tasks.register("checkSharedUiBoundary") {
    group = "verification"
    description = "Fails when shared Compose UI depends on Android or Android-only application layers."

    doLast {
        val forbiddenImports = listOf(
            "import android.",
            "import androidx.media3.",
            "import com.fluxa.app.data.",
            "import com.fluxa.app.player."
        )
        val violations = fileTree("shared/src/commonMain") {
            include("**/*.kt")
        }.files.flatMap { file ->
            val text = file.readText()
            forbiddenImports
                .filter { forbidden -> text.contains(forbidden) }
                .map { forbidden -> "${file.relativeTo(rootDir)} must not depend on $forbidden" }
        }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }
    }
}

tasks.register("checkAppleTypedCatalogBridge") {
    group = "verification"
    description = "Fails when the Apple catalog bridge falls back to JSON or notification handoffs."

    doLast {
        val requiredFiles = mapOf(
            "shared/src/iosMain/kotlin/com/fluxa/app/shared/platform/AppleCatalogHomeDataSource.kt" to listOf(
                "AppleCatalogHomeSnapshot",
                "setOnRefreshRequested",
                "fun update(snapshot: AppleCatalogHomeSnapshot)"
            ),
            "appleApp/iOS/FluxaAppleCatalogStartup.swift" to listOf(
                "AppleCatalogHomeSnapshot",
                "AppleCatalogRowSnapshot",
                "AppleCatalogItemSnapshot"
            ),
            "appleApp/iOS/FluxaIosApp.swift" to listOf("setCatalogHomeRefreshHandler")
        )
        val forbiddenTokens = listOf(
            "updateCatalogHomeJson",
            "FluxaAppleCatalogRefreshRequested",
            "NSNotificationCenter",
            "NotificationCenter.default.addObserver"
        )
        val violations = requiredFiles.flatMap { (relativePath, requiredTokens) ->
            val file = rootProject.file(relativePath)
            if (!file.exists()) {
                return@flatMap listOf("$relativePath is missing")
            }
            val text = file.readText()
            (requiredTokens.filterNot(text::contains).map { token ->
                "$relativePath must contain $token"
            } + forbiddenTokens.filter(text::contains).map { token ->
                "$relativePath must not contain $token"
            })
        }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }

        val bridgeFiles = fileTree("shared/src/iosMain/kotlin/com/fluxa/app/shared/platform") {
            include("Apple*DataSource.kt")
        }.files + fileTree("appleApp/iOS") {
            include("FluxaApple*Startup.swift", "FluxaIosApp.swift")
        }.files
        val legacyBridgeTokens = listOf(
            "updateDetailJson",
            "updateSearchJson",
            "updateDiscoverJson",
            "updateLibraryJson",
            "updateCalendarJson",
            "updateAddonStoreJson",
            "updateAuthJson",
            "NSNotificationCenter",
            "NotificationCenter.default.addObserver"
        )
        val legacyViolations = bridgeFiles.flatMap { file ->
            val text = file.readText()
            legacyBridgeTokens.filter(text::contains).map { token ->
                "${file.relativeTo(rootDir)} must not contain $token"
            }
        }
        if (legacyViolations.isNotEmpty()) {
            throw GradleException(legacyViolations.joinToString("\n"))
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

        val urlFacade = rootProject.file("data/src/main/java/com/fluxa/app/domain/discovery/StremioAddonUrls.kt")
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

tasks.register("checkFluxaCoreJniSymbols") {
    group = "verification"
    description = "Fails when FluxaCoreNative declares JNI methods not exported by fluxa_core."
    dependsOn("buildFluxaCoreHost")

    doLast {
        val nativeFile = rootProject.file("data/src/main/java/com/fluxa/app/core/rust/FluxaCoreNative.kt")
        val libraryFile = rustCoreProjectDir.resolve("target/debug/$rustHostLibraryName")
        if (!nativeFile.exists()) {
            throw GradleException("${nativeFile.relativeTo(rootDir)} is missing")
        }
        if (!libraryFile.exists()) {
            throw GradleException("Rust build did not produce ${libraryFile.absolutePath}")
        }

        val declaredMethods = Regex("""private\s+external\s+fun\s+([A-Za-z0-9_]+)\s*\(""")
            .findAll(nativeFile.readText())
            .map { it.groupValues[1] }
            .toSortedSet()
        val expectedSymbols = declaredMethods
            .map { method -> "Java_com_fluxa_app_core_rust_FluxaCoreNative_$method" }
            .toSortedSet()

        val nmTools = listOf("llvm-nm", "nm")
        val symbolOutput = nmTools.firstNotNullOfOrNull { tool ->
            runCatching {
                val process = ProcessBuilder(tool, "-g", libraryFile.absolutePath)
                    .redirectErrorStream(true)
                    .start()
                val output = process.inputStream.bufferedReader().readText()
                if (process.waitFor() == 0) output else null
            }
                .getOrNull()
        } ?: throw GradleException("Could not run llvm-nm or nm to inspect ${libraryFile.absolutePath}")

        val exportedSymbols = symbolOutput
            .lineSequence()
            .flatMap { line -> line.trim().split(Regex("""\s+""")).asSequence() }
            .map { token -> token.trimStart('_') }
            .filter { token -> token.startsWith("Java_com_fluxa_app_core_rust_FluxaCoreNative_") }
            .toSet()

        val missing = expectedSymbols.filterNot { symbol -> symbol in exportedSymbols }
        if (missing.isNotEmpty()) {
            throw GradleException(
                "FluxaCoreNative declares JNI methods missing from fluxa_core:\n${missing.joinToString("\n")}"
            )
        }
    }
}


tasks.register("checkFluxaStreamingJniSymbols") {
    group = "verification"
    description = "Fails when FluxaStreamingNative declares JNI methods not exported by fluxa_streaming_engine."
    dependsOn("buildFluxaStreamingEngineHost")

    doLast {
        val nativeFile = rootProject.file("player/src/main/java/com/fluxa/app/core/rust/FluxaStreamingNative.kt")
        val libraryFile = rustCoreProjectDir.resolve("fluxa-streaming-engine/target/debug/$rustStreamingHostLibraryName")
        if (!nativeFile.exists()) {
            throw GradleException("${nativeFile.relativeTo(rootDir)} is missing")
        }
        if (!libraryFile.exists()) {
            throw GradleException("Rust build did not produce ${libraryFile.absolutePath}")
        }

        val declaredMethods = Regex("""private\s+external\s+fun\s+([A-Za-z0-9_]+)\s*\(""")
            .findAll(nativeFile.readText())
            .map { it.groupValues[1] }
            .toSortedSet()
        val expectedSymbols = declaredMethods
            .map { method -> "Java_com_fluxa_app_core_rust_FluxaStreamingNative_$method" }
            .toSortedSet()

        val nmTools = listOf("llvm-nm", "nm")
        val symbolOutput = nmTools.firstNotNullOfOrNull { tool ->
            runCatching {
                val process = ProcessBuilder(tool, "-g", libraryFile.absolutePath)
                    .redirectErrorStream(true)
                    .start()
                val output = process.inputStream.bufferedReader().readText()
                if (process.waitFor() == 0) output else null
            }
                .getOrNull()
        } ?: throw GradleException("Could not run llvm-nm or nm to inspect ${libraryFile.absolutePath}")

        val exportedSymbols = symbolOutput
            .lineSequence()
            .flatMap { line -> line.trim().split(Regex("""\s+""")).asSequence() }
            .map { token -> token.trimStart('_') }
            .filter { token -> token.startsWith("Java_com_fluxa_app_core_rust_FluxaStreamingNative_") }
            .toSet()

        val missing = expectedSymbols.filterNot { symbol -> symbol in exportedSymbols }
        if (missing.isNotEmpty()) {
            throw GradleException(
                "FluxaStreamingNative declares JNI methods missing from fluxa_streaming_engine:\n${missing.joinToString("\n")}"
            )
        }
    }
}

tasks.register("qualityCheck") {
    group = "verification"
    description = "Runs the default local quality gate for Fluxa."
    dependsOn(
        "checkKotlinFileSize",
        "checkSharedUiBoundary",
        "checkAppleTypedCatalogBridge",
        "checkRustCoreBoundary",
        "checkFluxaCoreJniSymbols",
        "checkFluxaStreamingJniSymbols",
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

tasks.register<Exec>("buildFluxaStreamingEngineHost") {
    group = "build"
    description = "Builds the Fluxa streaming engine for the host toolchain."
    workingDir = rustCoreProjectDir.resolve("fluxa-streaming-engine")
    commandLine("cargo", "build")
}

tasks.register("buildFluxaCoreAndroid") {
    group = "build"
    description = "Builds the Fluxa Rust core for Android JNI ABIs."
    dependsOn(":app:buildFluxaCore")
}
