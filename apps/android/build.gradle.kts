import org.gradle.api.GradleException
import org.gradle.api.tasks.Exec

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.test) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.compose.multiplatform) apply false
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.hilt) apply false
}

val maxKotlinFileLines = 1500
val rustCoreProjectDir = rootProject.layout.projectDirectory.asFile.resolve("../../core/fluxa-core").canonicalFile
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
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/discovery/StremioAddonUrls.kt" to listOf(
        "FluxaCoreNative.normalizeManifestUrl",
        "FluxaCoreNative.identity",
        "FluxaCoreNative.manifestCandidates",
        "FluxaCoreNative.baseUrl",
        "FluxaCoreNative.preferHttpsAssetUrl"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/discovery/StremioAddonProtocol.kt" to listOf(
        "FluxaCoreNative.supportsResource"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/core/StremioId.kt" to listOf(
        "FluxaCoreNative.parseEpisodeLocator",
        "FluxaCoreNative.streamRequestIds"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/stream/StreamPlaybackResolver.jvm.kt" to listOf(
        "FluxaCoreNative.streamPlaybackInfo"
    ),
    "player/src/androidMain/kotlin/com/fluxa/app/player/TorrentStreamManager.kt" to listOf(
        "TorrentCorePolicy.plan",
        "TorrentCorePolicy.statusInfo"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/repository/StremioAddonManifestClient.kt" to listOf(
        "FluxaCoreNative.buildResourceUrl",
        "FluxaCoreNative.manifestFetchPlan",
        "FluxaCoreNative.parseManifestJson",
        "FluxaCoreNative.resolveManifestAssets",
        "FluxaCoreNative.mergeLiveManifest"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/repository/StremioAddonResourceClient.kt" to listOf(
        "FluxaCoreNative.parseAndPlanAddonResource",
        "FluxaCoreNative.parseExtraArgs"
    ),
    "player/src/androidMain/kotlin/com/fluxa/app/player/TorrentServerEngine.kt" to listOf(
        "FluxaStreamingNative.startTorrentServer",
        "FluxaStreamingNative.stopTorrentServer"
    ),
    "player/src/androidMain/kotlin/com/fluxa/app/player/TorrentCorePolicy.kt" to listOf(
        "FluxaCoreNative.torrentRuntimeInfo",
        "FluxaCoreNative.torrentStatusInfo"
    ),
    "app/src/main/java/com/fluxa/app/ui/catalog/AndroidStreamSourceSelectionPolicy.kt" to listOf(
        "FluxaCoreNative.selectStreamIndex"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/discovery/DiscoverCatalogContentLoader.kt" to listOf(
        "FluxaCoreNative.filterDiscoverResults",
        "FluxaCoreNative.discoverCatalogCacheKey"
    ),
    "app/src/main/java/com/fluxa/app/domain/discovery/StreamDiscovery.kt" to listOf(
        "FluxaCoreNative.streamDiscoveryExecutionPolicy"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/discovery/MetadataFeeds.kt" to listOf(
        "FluxaCoreNative.stableFeedPart",
        "FluxaCoreNative.toggleMetadataFeed",
        "FluxaCoreNative.setMetadataFeedGroupEnabled",
        "FluxaCoreNative.moveMetadataFeedOrder"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/ContentIdentity.kt" to listOf(
        "FluxaCoreNative.contentTraktKey",
        "FluxaCoreNative.contentMergeKeys",
        "FluxaCoreNative.contentWatchedKeysBatch"
    ),
    "core/src/commonMain/kotlin/com/fluxa/app/core/rust/FluxaHeadlessEffectRunner.kt" to listOf(
        "FluxaHeadlessEngine",
        "HeadlessPlatformEnvironment"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/core/rust/FluxaCoreNative.kt" to listOf(
        "NativeCoreCapabilitySet",
        "FluxaCoreUniFfi.coreInvokeValue"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/core/rust/FluxaCoreUniFfi.kt" to listOf(
        "com.fluxa.core.uniffi",
        "FluxaHeadlessEngine"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/repository/TraktIntegration.kt" to listOf(
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
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/repository/StremioRepository.kt" to listOf(
        "FluxaCoreNative.libraryContinueWatchingItems",
        "FluxaCoreNative.watchedVideoIds",
        "FluxaCoreNative.playbackProgressItem",
        "FluxaCoreNative.clearPlaybackProgressItem",
        "FluxaCoreNative.watchedStateItems",
        "FluxaCoreNative.traktHistoryRequest"
    ),
    "data/src/jvmCommonMain/kotlin/com/fluxa/app/data/local/ProfileManager.kt" to listOf(
        "FluxaCoreNative.sanitizeProfile",
        "FluxaCoreNative.profileLocalAddonsKey"
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

tasks.register("checkKmpCommonBoundary") {
    group = "verification"
    description = "Fails when KMP common source sets depend on platform-only implementation APIs."

    doLast {
        val forbiddenImports = listOf(
            "import android.",
            "import androidx.lifecycle.",
            "import androidx.media3.",
            "import androidx.work.",
            "import com.google.gson.",
            "import dagger.",
            "import javax.inject.",
            "import okhttp3.",
            "import retrofit2.",
            "import java."
        )
        val sourceRoots = listOf("core", "data", "player")
        val violations = sourceRoots.flatMap { module ->
            fileTree("$module/src/commonMain") {
                include("**/*.kt")
            }.files.flatMap { file ->
                val text = file.readText()
                forbiddenImports
                    .filter(text::contains)
                    .map { forbidden -> "${file.relativeTo(rootDir)} must not depend on $forbidden" }
            }
        }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }
    }
}

tasks.register("checkSharedTransportModels") {
    group = "verification"
    description = "Fails when portable account and streaming models return to Android-only source sets."

    doLast {
        val forbiddenAndroidModels = listOf(
            "data/src/androidMain/kotlin/com/fluxa/app/data/remote/NuvioModels.kt",
            "data/src/androidMain/kotlin/com/fluxa/app/data/remote/StremioModels.kt",
            "data/src/androidMain/kotlin/com/fluxa/app/data/remote/TraktModels.kt",
            "data/src/androidMain/kotlin/com/fluxa/app/player/TorrentModels.kt"
        )
        val requiredCommonModels = listOf(
            "data/src/commonMain/kotlin/com/fluxa/app/data/remote/NuvioModels.kt",
            "data/src/commonMain/kotlin/com/fluxa/app/data/remote/StremioModels.kt",
            "data/src/commonMain/kotlin/com/fluxa/app/data/remote/TraktModels.kt",
            "data/src/commonMain/kotlin/com/fluxa/app/player/TorrentModels.kt"
        )
        val violations = forbiddenAndroidModels.filter { rootProject.file(it).exists() }
            .map { "$it must not exist" } +
            requiredCommonModels.filterNot { rootProject.file(it).exists() }
                .map { "$it is required" }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }
    }
}

tasks.register("checkLegacySourceSets") {
    group = "verification"
    description = "Fails when legacy Android compatibility source trees or mappings return."

    doLast {
        val violations = mutableListOf<String>()
        listOf("data", "player").forEach { module ->
            val legacyRoot = rootProject.file("$module/src/main/java")
            if (legacyRoot.exists() && legacyRoot.walkTopDown().any { it.isFile }) {
                violations += "$module/src/main/java must remain empty"
            }
            val buildFile = rootProject.file("$module/build.gradle.kts").readText()
            if (buildFile.contains("src/main/java")) {
                violations += "$module/build.gradle.kts must not map src/main/java"
            }
        }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }
    }
}

tasks.register("checkAppleTvosKmpBoundary") {
    group = "verification"
    description = "Fails when tvOS links Compose UI or duplicates shared Stremio protocol logic."

    doLast {
        val projectText = rootProject.file("../apple/project.yml").readText()
        val handlerText = rootProject.file("../apple/tvOS/FluxaTvosEffectHandler.swift").readText()
        val catalogServiceText = rootProject.file("../apple/AppleCore/FluxaAppleCatalogService.swift").readText()
        val requiredProjectTokens = listOf(
            ":core:embedAndSignAppleFrameworkForXcode",
            ":data:embedAndSignAppleFrameworkForXcode",
            ":player:embedAndSignAppleFrameworkForXcode"
        )
        val requiredHandlerTokens = listOf(
            "import FluxaCore",
            "FluxaAppleCatalogService",
            "loadHomeRows"
        )
        val requiredCatalogServiceTokens = listOf(
            "FluxaAppleAddonCatalogResolver",
            "FluxaAppleCatalogLoader",
            "resolveRequests",
            "loadRows"
        )
        val violations = requiredProjectTokens.filterNot(projectText::contains).map { token ->
            "../apple/project.yml must contain $token"
        } + requiredHandlerTokens.filterNot(handlerText::contains).map { token ->
            "../apple/tvOS/FluxaTvosEffectHandler.swift must contain $token"
        } + requiredCatalogServiceTokens.filterNot(catalogServiceText::contains).map { token ->
            "../apple/AppleCore/FluxaAppleCatalogService.swift must contain $token"
        } + listOf("import FluxaShared", "JSONDecoder()", "struct Stremio", "FluxaCoreStremio").filter(handlerText::contains).map { token ->
            "../apple/tvOS/FluxaTvosEffectHandler.swift must not contain $token"
        }
        if (violations.isNotEmpty()) {
            throw GradleException(violations.joinToString("\n"))
        }
    }
}

tasks.register("checkRustCoreBoundary") {
    group = "verification"
    description = "Fails when platform-independent core behavior stops delegating to ../../core/fluxa-core."

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

        val urlFacade = rootProject.file("data/src/jvmCommonMain/kotlin/com/fluxa/app/domain/discovery/StremioAddonUrls.kt")
        val duplicatedUrlLogic = if (urlFacade.exists()) {
            val text = urlFacade.readText()
            listOf("http://", "https://", "stremio://", "manifest.json", "Regex(")
                .filter { token -> text.contains(token) }
                .map { token -> "${urlFacade.relativeTo(rootDir)} must not reimplement URL rules containing `$token`" }
        } else {
            emptyList()
        }

        val viewModelBackendRules = mapOf(
            "app/src/main/java/com/fluxa/app/ui/catalog/DetailViewModel.kt" to listOf(
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
        val nativeFile = rootProject.file("data/src/jvmCommonMain/kotlin/com/fluxa/app/core/rust/FluxaCoreNative.kt")
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
        val nativeFile = rootProject.file("player/src/androidMain/kotlin/com/fluxa/app/core/rust/FluxaStreamingNative.kt")
        val libraryFile = rustCoreProjectDir.resolve("target/debug/$rustStreamingHostLibraryName")
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
        "checkKmpCommonBoundary",
        "checkSharedTransportModels",
        "checkLegacySourceSets",
        "checkAppleTvosKmpBoundary",
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
