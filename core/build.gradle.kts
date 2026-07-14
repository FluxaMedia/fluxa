import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
    alias(libs.plugins.fluxa.kmp.library)
}

android {
    namespace = "com.fluxa.app.core"
    sourceSets.getByName("main").assets.srcDir("src/commonMain/resources")
}

kotlin {
    tvosArm64()
    tvosSimulatorArm64()

    targets
        .withType<KotlinNativeTarget>()
        .matching { it.name.startsWith("tvos") }
        .configureEach {
            binaries.framework {
                baseName = "FluxaCore"
                isStatic = true
            }
        }

    sourceSets {
        commonMain.dependencies {
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.kotlinx.datetime)
        }
        androidMain.dependencies {
            implementation(libs.androidx.core.ktx)
            implementation(libs.kotlinx.coroutines.android)
        }
    }
}
