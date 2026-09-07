import java.util.Properties
import org.gradle.api.GradleException

plugins {
    alias(libs.plugins.fluxa.android.application)
    alias(libs.plugins.fluxa.android.compose)
    alias(libs.plugins.fluxa.android.hilt)
    alias(libs.plugins.fluxa.android.rust)
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
        minSdk = 24
        targetSdk = 36
        versionCode = 700
        versionName = "2.1.7"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        buildConfigField("String", "TRAKT_CLIENT_ID", "\"${secret("TRAKT_CLIENT_ID")}\"")
        buildConfigField("String", "TRAKT_CLIENT_SECRET", "\"${secret("TRAKT_CLIENT_SECRET")}\"")
        buildConfigField("String", "SIMKL_CLIENT_ID", "\"${secret("SIMKL_CLIENT_ID")}\"")
        buildConfigField("String", "ANILIST_CLIENT_ID", "\"${secret("ANILIST_CLIENT_ID")}\"")
        buildConfigField("String", "NUVIO_SUPABASE_URL", "\"${secret("FLUXA_NUVIO_SUPABASE_URL")}\"")
        buildConfigField("String", "NUVIO_SUPABASE_KEY", "\"${secret("FLUXA_NUVIO_SUPABASE_KEY")}\"")
        buildConfigField("String", "FLUXA_SYNC_BASE_URL", "\"${secret("FLUXA_SYNC_BASE_URL")}\"")

    }

    testOptions {
        unitTests.isReturnDefaultValues = true
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
            // libmpv is the lowest common native playback floor for mobile and TV.
            minSdk = 26
            applicationId = "com.fluxa.app.mobile"
            buildConfigField("String", "DEVICE_FLAVOR", "\"mobile\"")
            buildConfigField("Boolean", "IS_TV", "false")
        }
        create("tv") {
            dimension = "device"
            // libmpv requires API 26; Android TV 7.0/7.1 (API 24-25) devices are unsupported.
            minSdk = 26
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
        create("benchmark") {
            initWith(getByName("release"))
            signingConfig = signingConfigs.getByName("debug")
            matchingFallbacks += listOf("release")
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
    implementation(libs.androidx.documentfile)
    testImplementation(libs.jna)
    testImplementation(libs.okhttp.mockwebserver)
    implementation(libs.androidx.activity.compose)
    implementation(libs.bundles.coroutines)
    implementation(libs.bundles.lifecycle)
    implementation(libs.androidx.work.runtime)
    implementation(libs.androidx.hilt.lifecycle.viewmodel.compose)
    implementation(libs.androidx.hilt.work)
    ksp(libs.androidx.hilt.compiler)

    // Compose
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.foundation)
    debugImplementation(libs.androidx.compose.ui.tooling)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.runtime.tracing)

    // TV
    implementation(libs.androidx.tv.material)

    // Image loading
    implementation(libs.bundles.coil3)

    // Room
    implementation(libs.androidx.room.runtime)

    // Media3 (for ExoPlayer access and @UnstableApi)
    implementation(libs.bundles.media3)
    implementation(libs.media3.session)

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
