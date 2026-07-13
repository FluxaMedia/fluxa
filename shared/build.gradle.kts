import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
    alias(libs.plugins.fluxa.kmp.compose)
}

android {
    namespace = "com.fluxa.app.shared"
}

kotlin {
    targets
        .withType<KotlinNativeTarget>()
        .matching { it.name.startsWith("ios") }
        .configureEach {
            binaries.framework {
                baseName = "FluxaShared"
                isStatic = true
                export(project(":core"))
            }
        }

    sourceSets {
        commonMain.dependencies {
            api(project(":core"))
            implementation(libs.coil3)
            implementation(libs.coil3.compose)
            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.androidx.lifecycle.viewmodel)
        }
    }
}
