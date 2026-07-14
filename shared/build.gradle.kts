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
                export(project(":data"))
            }
        }

    sourceSets {
        commonMain.dependencies {
            api(project(":core"))
            api(project(":data"))
            api(project(":player"))
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.datetime)
        }
        androidMain.dependencies {
            implementation(libs.coil3)
            implementation(libs.coil3.compose)
            implementation(libs.coil3.network.okhttp)
        }
    }
}
