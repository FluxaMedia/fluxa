import java.util.Properties

plugins {
    alias(libs.plugins.fluxa.kmp.library)
    alias(libs.plugins.fluxa.android.hilt)
    alias(libs.plugins.ksp)
    alias(libs.plugins.kotlin.serialization)
}

val localProperties = Properties().apply {
    val localFile = rootProject.file("local.properties")
    if (localFile.exists()) localFile.inputStream().use { load(it) }
}

fun secret(name: String, default: String = ""): String =
    providers.gradleProperty(name).orNull
        ?: System.getenv(name)
        ?: localProperties.getProperty(name, default)

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
    arg("room.incremental", "true")
    arg("room.generateKotlin", "true")
}

android {
    namespace = "com.fluxa.app.data"
    buildFeatures { buildConfig = true }

    defaultConfig {
        buildConfigField("String", "TRAKT_CLIENT_ID", "\"${secret("TRAKT_CLIENT_ID")}\"")
        buildConfigField("String", "TRAKT_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "MAL_CLIENT_ID", "\"${secret("MAL_CLIENT_ID")}\"")
        buildConfigField("String", "MAL_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "SIMKL_CLIENT_ID", "\"${secret("SIMKL_CLIENT_ID")}\"")
        buildConfigField("String", "SIMKL_CLIENT_SECRET", "\"\"")
        buildConfigField("String", "ANILIST_CLIENT_ID", "\"${secret("ANILIST_CLIENT_ID")}\"")
        buildConfigField("String", "ANILIST_CLIENT_SECRET", "\"${secret("ANILIST_CLIENT_SECRET")}\"")
    }
}

kotlin {
    tvosArm64()
    tvosSimulatorArm64()

    sourceSets {
        commonMain {
            dependencies {
                implementation(libs.kotlinx.serialization.json)
            }
        }
        androidMain {
            kotlin.srcDir("src/main/java")
            dependencies {
                implementation(project(":core"))
                implementation(libs.androidx.core.ktx)
                implementation(libs.bundles.coroutines)
                implementation(libs.androidx.work.runtime)
                implementation(libs.androidx.hilt.work)
                implementation(libs.androidx.room.runtime)
                implementation(libs.androidx.room.ktx)
                implementation(libs.bundles.retrofit)
                implementation(libs.okhttp.logging)
                implementation(libs.okhttp.doh)
            }
        }
    }
}

dependencies {
    add("kspAndroid", libs.androidx.hilt.compiler)
    add("kspAndroid", libs.androidx.room.compiler)
}
