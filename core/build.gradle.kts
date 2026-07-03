plugins {
    alias(libs.plugins.fluxa.android.library)
}

android {
    namespace = "com.fluxa.app.core"
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.bundles.coroutines)
}
