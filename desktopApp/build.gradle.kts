import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("org.jetbrains.compose")
    id("org.jetbrains.kotlin.plugin.compose")
}

kotlin {
    jvm("desktop") {
        compilerOptions { jvmTarget.set(JvmTarget.JVM_17) }
    }

    sourceSets {
        val desktopMain by getting {
            dependencies {
                implementation(project(":data"))
                implementation(project(":shared"))
                implementation(libs.kotlinx.coroutines.core)
                implementation(libs.gson)
                implementation(libs.okhttp)
                implementation(compose.desktop.currentOs)
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material3)
                implementation(compose.ui)
            }
        }
    }
}

compose.desktop {
    application {
        mainClass = "com.fluxa.app.desktop.MainKt"
    }
}

val rustCoreLibraryDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core/target/debug")
val rustCoreLibraryPath = rustCoreLibraryDir.resolve(
    when {
        org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "libfluxa_core.dylib"
        org.gradle.internal.os.OperatingSystem.current().isWindows -> "fluxa_core.dll"
        else -> "libfluxa_core.so"
    }
).absolutePath

tasks.matching { it.name == "run" || it.name == "runDistributable" || it.name == "hotRunDesktop" }.configureEach {
    dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
    if (this is JavaExec) {
        systemProperty("fluxa.core.library.path", rustCoreLibraryPath)
        // The UniFFI/JNA binding path resolves the library independently of
        // System.load, via JNA's own search (jna.library.path).
        systemProperty("jna.library.path", rustCoreLibraryDir.absolutePath)
    }
}
