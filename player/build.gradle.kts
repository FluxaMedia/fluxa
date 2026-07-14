plugins {
    alias(libs.plugins.fluxa.kmp.library)
    alias(libs.plugins.fluxa.android.hilt)
    alias(libs.plugins.ksp)
}

android {
    namespace = "com.fluxa.app.player"

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            implementation(libs.kotlinx.coroutines.core)
        }
        androidMain {
            kotlin.srcDir("src/main/java")
            dependencies {
                implementation(project(":core"))
                implementation(project(":data"))
                implementation(libs.androidx.core.ktx)
                implementation(libs.bundles.coroutines)
                implementation(libs.androidx.lifecycle.runtime)
                implementation(libs.bundles.media3)
                implementation(libs.jellyfin.media3)
                implementation(libs.mpv)
                implementation(libs.bundles.retrofit)
                implementation(libs.okhttp.logging)
            }
        }
    }
}
