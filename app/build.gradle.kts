import java.util.Properties
import org.gradle.api.GradleException
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.testing.Test

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("com.google.devtools.ksp")
    id("org.jetbrains.kotlin.plugin.compose")
    id("com.google.dagger.hilt.android")
}

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
    arg("room.incremental", "true")
    arg("room.generateKotlin", "true")
}

val localProperties = Properties().apply {
    val localFile = rootProject.file("local.properties")
    if (localFile.exists()) {
        localFile.inputStream().use { load(it) }
    }
}

fun secret(name: String, default: String = ""): String {
    return providers.gradleProperty(name).orNull
        ?: System.getenv(name)
        ?: localProperties.getProperty(name, default)
}

android {
    namespace = "com.fluxa.app"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.fluxa.app"
        minSdk = 30
        targetSdk = 35
        versionCode = 700
        versionName = "2.1.7"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        buildConfigField("String", "TRAKT_CLIENT_ID", "\"${secret("TRAKT_CLIENT_ID")}\"")
        buildConfigField("String", "TRAKT_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "MAL_CLIENT_ID", "\"${secret("MAL_CLIENT_ID")}\"")
        buildConfigField("String", "MAL_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "SIMKL_CLIENT_ID", "\"${secret("SIMKL_CLIENT_ID")}\"")
        buildConfigField("String", "SIMKL_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "UPDATE_URL", "\"${secret("FLUXA_UPDATE_URL")}\"")

    }

    splits {
        abi {
            isEnable = true
            reset()
            include("arm64-v8a")
            isUniversalApk = false
        }
    }

    bundle {
        abi {
            enableSplit = true
        }
    }

    flavorDimensions += "device"
    productFlavors {
        create("mobile") {
            dimension = "device"
            applicationId = "com.fluxa.app.mobile"
            buildConfigField("String", "DEVICE_FLAVOR", "\"mobile\"")
            buildConfigField("Boolean", "IS_TV", "false")
        }
        create("tv") {
            dimension = "device"
            applicationId = "com.fluxa.app.tv"
            buildConfigField("String", "DEVICE_FLAVOR", "\"tv\"")
            buildConfigField("Boolean", "IS_TV", "true")
        }
    }

    val releaseStoreFile = secret("FLUXA_RELEASE_STORE_FILE")
    val releaseStorePassword = secret("FLUXA_RELEASE_STORE_PASSWORD")
    val releaseKeyAlias = secret("FLUXA_RELEASE_KEY_ALIAS")
    val releaseKeyPassword = secret("FLUXA_RELEASE_KEY_PASSWORD")
    val hasReleaseSigningCredentials = listOf(
        releaseStoreFile,
        releaseStorePassword,
        releaseKeyAlias,
        releaseKeyPassword
    ).all { it.isNotBlank() }

    signingConfigs {
        if (hasReleaseSigningCredentials) {
            create("release") {
                storeFile = rootProject.file(releaseStoreFile)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            if (hasReleaseSigningCredentials) {
                signingConfig = signingConfigs.getByName("release")
            } else {
                signingConfig = signingConfigs.getByName("debug")
            }
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlin {
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    composeCompiler {
        enableStrongSkippingMode = true
    }
    packaging {
        jniLibs {
            useLegacyPackaging = false
            pickFirsts.add("**/*.so")
        }
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDir(layout.buildDirectory.dir("generated/rustJniLibs"))
            java.srcDir(layout.buildDirectory.dir("generated/source/uniffi/main/kotlin"))
        }
    }
}

val rustCrateDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core").canonicalFile
val rustStreamingCrateDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-streaming-engine").canonicalFile
val rustJniOutputDir = layout.buildDirectory.dir("generated/rustJniLibs")
val androidNdkDir = providers.environmentVariable("ANDROID_NDK_HOME").orElse(
    providers.environmentVariable("ANDROID_NDK_ROOT").orElse(
        providers.provider {
            val localSdk = rootProject.file("local.properties")
                .takeIf { it.exists() }
                ?.readLines()
                ?.firstOrNull { it.startsWith("sdk.dir=") }
                ?.substringAfter("sdk.dir=")
            listOfNotNull(
                localSdk?.let { "$it/ndk/28.2.13676358" },
                "${System.getProperty("user.home")}/Android/Sdk/ndk/28.2.13676358",
                "${System.getProperty("user.home")}/Android/Sdk/ndk/27.1.12297006"
            ).firstOrNull { file(it).exists() }.orEmpty()
        }
    )
)

val rustAndroidTargets = listOf(
    Triple("arm64-v8a", "aarch64-linux-android", "AARCH64_LINUX_ANDROID"),
)

val rustProfile = if (gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }) "release" else "debug"
val rustCargoProfileArgs = if (rustProfile == "release") listOf("--release") else emptyList()
val rustHostTag = when {
    org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "darwin-x86_64"
    org.gradle.internal.os.OperatingSystem.current().isWindows -> "windows-x86_64"
    else -> "linux-x86_64"
}
val rustHostLibraryName = when {
    org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "libfluxa_core.dylib"
    org.gradle.internal.os.OperatingSystem.current().isWindows -> "fluxa_core.dll"
    else -> "libfluxa_core.so"
}

val rustTargetTasks = rustAndroidTargets.map { (abi, target, envName) ->
    val taskName = "buildFluxaCore${abi.split("-", "_").joinToString("") { it.replaceFirstChar(Char::uppercaseChar) }}"
    tasks.register<Exec>(taskName) {
        group = "build"
        description = "Builds the Fluxa Rust core for $abi."
        workingDir = rustCrateDir
        val linkerName = when (target) {
            "armv7-linux-androideabi" -> "armv7a-linux-androideabi26-clang"
            else -> "${target}26-clang"
        }
        commandLine("cargo", "build", "--target", target, *rustCargoProfileArgs.toTypedArray())
        inputs.files(fileTree(rustCrateDir) { exclude("target/**") })
        outputs.file(rustJniOutputDir.map { it.file("$abi/libfluxa_core.so") })
        doFirst {
            val ndkDir = androidNdkDir.get()
            if (ndkDir.isBlank() || !file(ndkDir).exists()) {
                throw GradleException("Android NDK is required to build ../fluxa-core.")
            }
            val toolchainBin = file("$ndkDir/toolchains/llvm/prebuilt/$rustHostTag/bin")
            val clang = file(toolchainBin.resolve(linkerName)).absolutePath
            environment("CARGO_TARGET_${envName}_LINKER", clang)
            environment("CC_$target", clang)
            environment("CC_${target.replace("-", "_")}", clang)
            environment("AR_${envName}", file(toolchainBin.resolve("llvm-ar")).absolutePath)
        }
        doLast {
            val builtLibrary = file("$rustCrateDir/target/$target/$rustProfile/libfluxa_core.so")
            if (!builtLibrary.exists()) {
                throw GradleException("Rust build did not produce ${builtLibrary.absolutePath}")
            }
            copy {
                from(builtLibrary)
                into(rustJniOutputDir.get().dir(abi))
            }
        }
    }
}

tasks.register("buildFluxaCore") {
    group = "build"
    description = "Builds the Fluxa Rust core for Android JNI ABIs."
    dependsOn(rustTargetTasks)
}

val rustStreamingTargetTasks = rustAndroidTargets.map { (abi, target, envName) ->
    val taskName = "buildFluxaStreamingEngine${abi.split("-", "_").joinToString("") { it.replaceFirstChar(Char::uppercaseChar) }}"
    tasks.register<Exec>(taskName) {
        group = "build"
        description = "Builds the Fluxa streaming engine for $abi."
        workingDir = rustStreamingCrateDir
        val linkerName = when (target) {
            "armv7-linux-androideabi" -> "armv7a-linux-androideabi26-clang"
            else -> "${target}26-clang"
        }
        commandLine("cargo", "build", "--target", target, *rustCargoProfileArgs.toTypedArray())
        inputs.files(fileTree(rustStreamingCrateDir) { exclude("target/**") })
        outputs.file(rustJniOutputDir.map { it.file("$abi/libfluxa_streaming_engine.so") })
        doFirst {
            val ndkDir = androidNdkDir.get()
            if (ndkDir.isBlank() || !file(ndkDir).exists()) {
                throw GradleException("Android NDK is required to build fluxa-streaming-engine.")
            }
            val toolchainBin = file("$ndkDir/toolchains/llvm/prebuilt/$rustHostTag/bin")
            val clang = file(toolchainBin.resolve(linkerName)).absolutePath
            environment("CARGO_TARGET_${envName}_LINKER", clang)
            environment("CC_$target", clang)
            environment("CC_${target.replace("-", "_")}", clang)
            environment("AR_${envName}", file(toolchainBin.resolve("llvm-ar")).absolutePath)
        }
        doLast {
            val builtLibrary = file("$rustStreamingCrateDir/target/$target/$rustProfile/libfluxa_streaming_engine.so")
            if (!builtLibrary.exists()) {
                throw GradleException("Rust build did not produce ${builtLibrary.absolutePath}")
            }
            copy {
                from(builtLibrary)
                into(rustJniOutputDir.get().dir(abi))
            }
        }
    }
}

tasks.register("buildFluxaStreamingEngine") {
    group = "build"
    description = "Builds the Fluxa streaming engine for Android JNI ABIs."
    dependsOn(rustStreamingTargetTasks)
}


val generateFluxaCoreUniFfiBindings by tasks.registering(Exec::class) {
    group = "build"
    description = "Generates Kotlin UniFFI bindings for the Fluxa Rust core."
    workingDir = rustCrateDir
    dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
    val outDir = layout.buildDirectory.dir("generated/source/uniffi/main/kotlin")
    commandLine(
        "cargo",
        "run",
        "--features",
        "uniffi-cli",
        "--bin",
        "uniffi-bindgen",
        "generate",
        "--library",
        "target/debug/$rustHostLibraryName",
        "--language",
        "kotlin",
        "--config",
        "uniffi.toml",
        "--out-dir",
        outDir.get().asFile.absolutePath
    )
    inputs.files(fileTree(rustCrateDir) { exclude("target/**") })
    outputs.dir(outDir)
}

tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn("buildFluxaCore")
    dependsOn("buildFluxaStreamingEngine")
    dependsOn(generateFluxaCoreUniFfiBindings)
}

tasks.withType<Test>().configureEach {
    dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
    dependsOn(generateFluxaCoreUniFfiBindings)
    systemProperty(
        "jna.library.path",
        rustCrateDir.resolve("target/debug").absolutePath
    )
    systemProperty(
        "fluxa.core.library.path",
        rustCrateDir.resolve("target/debug/$rustHostLibraryName").absolutePath
    )
}

val requestedTasks = gradle.startParameter.taskNames.map { it.lowercase() }
val requiresSignedRelease = requestedTasks.any { task ->
    "release" in task && listOf("bundle", "package", "install", "publish").any { keyword -> keyword in task }
}

if (requiresSignedRelease && !listOf(
        secret("FLUXA_RELEASE_STORE_FILE"),
        secret("FLUXA_RELEASE_STORE_PASSWORD"),
        secret("FLUXA_RELEASE_KEY_ALIAS"),
        secret("FLUXA_RELEASE_KEY_PASSWORD")
    ).all { it.isNotBlank() }
) {
    throw GradleException("Release signing credentials are required for release build tasks.")
}

dependencies {
    // AndroidX Core
    implementation("androidx.core:core-ktx:1.18.0")
    implementation("net.java.dev.jna:jna:5.17.0@aar")
    testImplementation("net.java.dev.jna:jna:5.17.0")
    implementation("androidx.activity:activity-compose:1.13.0")
    
    // Kotlin Coroutines
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.11.0")
    
    // Lifecycle & ViewModel
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.10.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
    implementation("androidx.work:work-runtime-ktx:2.11.2")
    implementation("com.google.dagger:hilt-android:2.58")
    ksp("com.google.dagger:hilt-compiler:2.58")
    implementation("androidx.hilt:hilt-navigation-compose:1.2.0")
    implementation("androidx.hilt:hilt-work:1.3.0")
    ksp("androidx.hilt:hilt-compiler:1.3.0")

    // Compose BOM — Kotlin 2.0.21 uyumlu
    val composeBom = platform("androidx.compose:compose-bom:2025.05.01")
    implementation(composeBom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.compose.foundation:foundation")
    debugImplementation("androidx.compose.ui:ui-tooling")
    implementation("androidx.compose.ui:ui-tooling-preview")
    // Composition tracing: emits composable-named slices in system traces. ~No cost when tracing is off.
    implementation("androidx.compose.runtime:runtime-tracing")

    // TV Material3
    implementation("androidx.tv:tv-foundation:1.0.0-alpha11")
    implementation("androidx.tv:tv-material:1.0.0")

    // Media3 (ExoPlayer)
    val media3_version = "1.10.1"
    implementation("androidx.media3:media3-common:$media3_version")
    implementation("androidx.media3:media3-exoplayer:$media3_version")
    implementation("androidx.media3:media3-exoplayer-hls:$media3_version")
    implementation("androidx.media3:media3-ui:$media3_version")
    implementation("androidx.media3:media3-datasource-okhttp:$media3_version")
    implementation("androidx.media3:media3-effect:$media3_version")
    implementation("org.jellyfin.media3:media3-ffmpeg-decoder:1.9.0+1")

    implementation("dev.jdtech.mpv:libmpv:1.0.0")

    // Network & Logging
    implementation("com.squareup.retrofit2:retrofit:3.0.0")
    implementation("com.squareup.retrofit2:converter-gson:3.0.0")
    implementation("com.squareup.okhttp3:logging-interceptor:5.3.2")
    implementation("com.squareup.okhttp3:okhttp-dnsoverhttps:5.3.2")
    // Coil 3: network backend is a separate artifact (okhttp fetcher required).
    val coil3_version = "3.5.0-beta01"
    implementation("io.coil-kt.coil3:coil:$coil3_version")
    implementation("io.coil-kt.coil3:coil-compose:$coil3_version")
    implementation("io.coil-kt.coil3:coil-network-okhttp:$coil3_version")
    implementation("io.coil-kt.coil3:coil-svg:$coil3_version")
    implementation("io.coil-kt.coil3:coil-gif:$coil3_version")
    implementation("org.jsoup:jsoup:1.22.2")

    // CloudStream extensions use Jackson at runtime.
    implementation("com.fasterxml.jackson.core:jackson-databind:2.17.0")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.17.0")

    // NewPipe Extractor — same artifact CloudStream brings transitively; explicit here for compile classpath
    implementation("com.github.teamnewpipe:NewPipeExtractor:v0.25.2")

    // CloudStream3 library - includes NiceHttp
    implementation("com.github.recloudstream.cloudstream:library:v4.7.0") {
        exclude(group = "org.mozilla", module = "rhino")
        exclude(group = "com.github.AmarullisVFX", module = "newpipeextractor")
        exclude(group = "com.github.AmaryllisVFX", module = "newpipeextractor")
        exclude(group = "com.github.AmaryllisVFX.newpipeextractor")
        exclude(group = "info.debatty", module = "java-string-similarity")
    }

    // QR Code Generation
    implementation("com.google.zxing:core:3.5.4")

    // Dynamic Theming
    implementation("androidx.palette:palette-ktx:1.0.0")

    // Local Integration Server (NanoHTTPD) for QR Login
    implementation("org.nanohttpd:nanohttpd:2.3.1")

    // Room Database
    val room_version = "2.8.4"
    implementation("androidx.room:room-runtime:$room_version")
    implementation("androidx.room:room-ktx:$room_version")
    ksp("androidx.room:room-compiler:$room_version")

    // Installs src/main/baseline-prof.txt into ART at first launch (AOT-compiles hot scroll classes).
    implementation("androidx.profileinstaller:profileinstaller:1.4.1")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test:runner:1.6.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
}
