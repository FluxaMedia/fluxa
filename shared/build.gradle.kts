plugins {
    alias(libs.plugins.fluxa.kmp.compose)
}

android {
    namespace = "com.fluxa.app.shared"
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            implementation(project(":core"))
            implementation(libs.coil3)
            implementation(libs.coil3.compose)
            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.androidx.lifecycle.viewmodel)
        }
    }
}
