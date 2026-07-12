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
        }
    }
}
