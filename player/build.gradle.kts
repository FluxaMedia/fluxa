import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

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
    tvosArm64()
    tvosSimulatorArm64()

    targets
        .withType<KotlinNativeTarget>()
        .matching { it.name.startsWith("ios") || it.name.startsWith("tvos") }
        .configureEach {
            binaries.framework {
                baseName = "FluxaPlayer"
                isStatic = true
            }
        }

    sourceSets {
        commonMain.dependencies {
            implementation(project(":data"))
            implementation(libs.kotlinx.coroutines.core)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
        }
        androidMain {
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
