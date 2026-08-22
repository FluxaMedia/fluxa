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
                export(project(":player"))
            }
        }

    sourceSets {
        val iosMain by creating {
            dependsOn(commonMain.get())
        }
        getByName("iosArm64Main").dependsOn(iosMain)
        getByName("iosSimulatorArm64Main").dependsOn(iosMain)
        commonMain.dependencies {
            api(project(":core"))
            api(project(":data"))
            api(project(":player"))
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.datetime)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.coil3)
            implementation(libs.coil3.compose)
            implementation(libs.haze)
        }
        val jvmCommonMain by creating {
            dependsOn(commonMain.get())
            dependencies {
                implementation(libs.okhttp)
                implementation(libs.jcifs.ng)
                implementation(libs.bouncycastle.bcprov)
            }
        }
        androidMain {
            dependsOn(jvmCommonMain)
            dependencies {
                implementation(libs.coil3.network.okhttp)
                implementation(libs.androidx.tv.foundation)
                implementation(libs.androidx.tv.material)
            }
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
        }
    }
}
