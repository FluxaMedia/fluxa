import java.util.Properties
import org.gradle.api.GradleException

plugins {
    alias(libs.plugins.fluxa.android.application)
    alias(libs.plugins.fluxa.android.rust)
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

val appVersionName = providers.gradleProperty("fluxaVersion").orNull
    ?: System.getenv("FLUXA_VERSION")
    ?: System.getenv("GITHUB_REF_NAME")?.removePrefix("v")
    ?: "1.0.0"

android {
    namespace = "com.fluxa.app"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.fluxa.app"
        minSdk = 24
        targetSdk = 36
        versionCode = 700
        versionName = appVersionName
    }

    splits {
        abi {
            isEnable = gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }
            reset()
            include("arm64-v8a", "armeabi-v7a", "x86", "x86_64")
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
    implementation(libs.androidx.core.ktx)
    implementation(libs.bundles.coroutines)
    implementation(libs.okhttp)
    implementation(libs.gson)
    implementation(libs.mpv)
}
