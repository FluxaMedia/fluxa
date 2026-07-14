plugins {
    alias(libs.plugins.fluxa.android.library)
    alias(libs.plugins.fluxa.android.compose)
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

dependencies {
    implementation(project(":core"))
    implementation(project(":data"))

    implementation(libs.androidx.core.ktx)
    implementation(libs.bundles.coroutines)
    implementation(libs.androidx.lifecycle.runtime)

    // Media3
    implementation(libs.bundles.media3)
    implementation(libs.jellyfin.media3)
    implementation(libs.mpv)

    // Network (TorrentServer API)
    implementation(libs.bundles.retrofit)
    implementation(libs.okhttp.logging)

    // Compose (player UI components)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.compose.material3)
}
