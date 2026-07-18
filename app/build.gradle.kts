import java.util.Properties
import org.gradle.api.GradleException
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.testing.Test

plugins {
    alias(libs.plugins.fluxa.android.application)
    alias(libs.plugins.fluxa.android.compose)
    alias(libs.plugins.fluxa.android.hilt)
    alias(libs.plugins.ksp)
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
        buildConfigField("String", "TRAKT_CLIENT_SECRET", "\"${secret("TRAKT_CLIENT_SECRET")}\"")
        buildConfigField("String", "MAL_CLIENT_ID", "\"${secret("MAL_CLIENT_ID")}\"")
        buildConfigField("String", "MAL_CLIENT_SECRET", "\"${secret("MAL_CLIENT_SECRET")}\"")
        buildConfigField("String", "SIMKL_CLIENT_ID", "\"${secret("SIMKL_CLIENT_ID")}\"")
        buildConfigField("String", "SIMKL_CLIENT_SECRET", "\"${secret("SIMKL_CLIENT_SECRET")}\"")
        buildConfigField("String", "ANILIST_CLIENT_ID", "\"${secret("ANILIST_CLIENT_ID")}\"")
        buildConfigField("String", "ANILIST_CLIENT_SECRET", "\"${secret("ANILIST_CLIENT_SECRET")}\"")
        buildConfigField("String", "NUVIO_SUPABASE_URL", "\"${secret("FLUXA_NUVIO_SUPABASE_URL", "https://api.nuvio.tv/")}\"")
        buildConfigField("String", "NUVIO_SUPABASE_KEY", "\"${secret("FLUXA_NUVIO_SUPABASE_KEY", "sb_publishable_1Clq8rlTVACkdcZuqr6_AD__xUUC_EN")}\"")

    }

    splits {
        abi {
            isEnable = gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }
            reset()
            include("arm64-v8a", "armeabi-v7a", "x86")
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
    buildFeatures {
        buildConfig = true
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
        }
    }
}

val rustCrateDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core").canonicalFile
val rustStreamingCrateDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core/fluxa-streaming-engine").canonicalFile
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
    Triple("armeabi-v7a", "armv7-linux-androideabi", "ARMV7_LINUX_ANDROIDEABI"),
    Triple("x86", "i686-linux-android", "I686_LINUX_ANDROID"),
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
        inputs.files(fileTree(rustCrateDir) {
            exclude("target/**", ".git/**", ".agents/**", ".codex/**", "fluxa-streaming-engine/target/**")
        })
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
        inputs.files(fileTree(rustStreamingCrateDir) {
            exclude("target/**", ".git/**", ".agents/**", ".codex/**")
        })
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
            val builtLibrary = file("$rustCrateDir/target/$target/$rustProfile/libfluxa_streaming_engine.so")
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


tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn("buildFluxaCore")
    dependsOn("buildFluxaStreamingEngine")
}

tasks.withType<Test>().configureEach {
    dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
    dependsOn(rootProject.tasks.named("buildFluxaStreamingEngineHost"))
    jvmArgs("-Djava.library.path=${rustCrateDir.resolve("target/debug").absolutePath}")
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
    // Submodules
    implementation(project(":core"))
    implementation(project(":shared"))
    implementation(project(":data"))
    implementation(project(":player"))

    implementation(libs.androidx.core.ktx)
    implementation("net.java.dev.jna:jna:${libs.versions.jna.get()}@aar")
    testImplementation(libs.jna)
    implementation(libs.androidx.activity.compose)
    implementation(libs.bundles.coroutines)
    implementation(libs.bundles.lifecycle)
    implementation(libs.androidx.work.runtime)
    implementation(libs.androidx.hilt.navigation.compose)
    implementation(libs.androidx.hilt.work)
    ksp(libs.androidx.hilt.compiler)

    // Compose
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.material.icons)
    implementation(libs.androidx.compose.foundation)
    debugImplementation(libs.androidx.compose.ui.tooling)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.runtime.tracing)

    // TV
    implementation(libs.androidx.tv.foundation)
    implementation(libs.androidx.tv.material)

    // Image loading
    implementation(libs.bundles.coil3)

    // Room
    implementation(libs.androidx.room.runtime)

    // Media3 (for ExoPlayer access and @UnstableApi)
    implementation(libs.bundles.media3)

    // Serialization / networking
    implementation(libs.retrofit.gson)
    implementation(libs.okhttp.logging)

    // CloudStream plugin host
    implementation(libs.cloudstream) {
        exclude(group = "org.mozilla", module = "rhino")
        exclude(group = "com.github.AmarullisVFX", module = "newpipeextractor")
        exclude(group = "com.github.AmaryllisVFX", module = "newpipeextractor")
        exclude(group = "com.github.AmaryllisVFX.newpipeextractor")
        exclude(group = "info.debatty", module = "java-string-similarity")
    }
    implementation(libs.jackson.databind)
    implementation(libs.jackson.module.kotlin)

    // Misc
    implementation(libs.zxing)
    implementation(libs.androidx.palette)
    implementation(libs.androidx.profileinstaller)
    implementation(libs.androidx.biometric)
    implementation(libs.androidx.browser)

    testImplementation(libs.junit)
    androidTestImplementation(libs.junit)
    androidTestImplementation(libs.androidx.test.runner)
    androidTestImplementation(libs.androidx.test.junit)
}
