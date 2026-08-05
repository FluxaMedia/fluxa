import java.util.Properties
import org.gradle.api.tasks.Exec
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
    alias(libs.plugins.fluxa.kmp.library)
    alias(libs.plugins.fluxa.android.hilt)
    alias(libs.plugins.ksp)
    alias(libs.plugins.kotlin.serialization)
}

val localProperties = Properties().apply {
    val localFile = rootProject.file("local.properties")
    if (localFile.exists()) localFile.inputStream().use { load(it) }
}

fun secret(name: String, default: String = ""): String =
    providers.gradleProperty(name).orNull
        ?: System.getenv(name)
        ?: localProperties.getProperty(name, default)

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
    arg("room.incremental", "true")
    arg("room.generateKotlin", "true")
}

android {
    namespace = "com.fluxa.app.data"
    buildFeatures { buildConfig = true }

    defaultConfig {
        buildConfigField("String", "TRAKT_CLIENT_ID", "\"${secret("TRAKT_CLIENT_ID")}\"")
        buildConfigField("String", "SIMKL_CLIENT_ID", "\"${secret("SIMKL_CLIENT_ID")}\"")
        buildConfigField("String", "ANILIST_CLIENT_ID", "\"${secret("ANILIST_CLIENT_ID")}\"")
    }
}

val rustCrateDir = rootProject.layout.projectDirectory.asFile.resolve("../fluxa-core").canonicalFile
val rustHostLibraryName = when {
    org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "libfluxa_core.dylib"
    org.gradle.internal.os.OperatingSystem.current().isWindows -> "fluxa_core.dll"
    else -> "libfluxa_core.so"
}
val uniffiKotlinOutDir = layout.buildDirectory.dir("generated/source/uniffi/main/kotlin")

val generateFluxaCoreUniFfiBindings by tasks.registering(Exec::class) {
    group = "build"
    description = "Generates Kotlin UniFFI bindings for the Fluxa Rust core."
    workingDir = rustCrateDir
    dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
    commandLine(
        "cargo",
        "run",
        "--features",
        "uniffi-cli",
        "--bin",
        "uniffi-bindgen",
        "generate",
        "--library",
        "target/debug/$rustHostLibraryName",
        "--language",
        "kotlin",
        "--config",
        "uniffi.toml",
        "--out-dir",
        uniffiKotlinOutDir.get().asFile.absolutePath
    )
    inputs.files(fileTree(rustCrateDir) {
        exclude("target/**", ".git/**", ".agents/**", ".codex/**", "fluxa-streaming-engine/target/**")
    })
    outputs.dir(uniffiKotlinOutDir)
}

tasks.matching { it.name == "preBuild" || it.name == "compileKotlinDesktop" || it.name == "kspKotlinDesktop" }.configureEach {
    dependsOn(generateFluxaCoreUniFfiBindings)
}

kotlin {
    tvosArm64()
    tvosSimulatorArm64()

    targets
        .withType<KotlinNativeTarget>()
        .matching { it.name.startsWith("tvos") }
        .configureEach {
            binaries.framework {
                baseName = "FluxaData"
                isStatic = true
            }
        }

    sourceSets {
        commonMain {
            dependencies {
                implementation(project(":core"))
                implementation(libs.kotlinx.serialization.json)
                implementation(libs.kotlinx.coroutines.core)
                api(libs.androidx.room.runtime)
            }
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
            implementation(libs.kotlinx.serialization.json)
        }
        val jvmCommonMain by creating {
            dependsOn(commonMain.get())
            kotlin.srcDir(uniffiKotlinOutDir)
            dependencies {
                implementation(libs.jna)
                implementation(libs.gson)
                implementation(libs.javax.inject)
                implementation(libs.okhttp)
                implementation(libs.okhttp.logging)
                implementation(libs.bundles.retrofit)
            }
        }
        androidMain {
            dependsOn(jvmCommonMain)
            dependencies {
                implementation("net.java.dev.jna:jna:${libs.versions.jna.get()}@aar")
                implementation(libs.androidx.core.ktx)
                implementation(libs.bundles.coroutines)
                implementation(libs.androidx.work.runtime)
                implementation(libs.androidx.hilt.work)
                implementation(libs.okhttp.doh)
            }
        }
        val desktopMain by getting {
            dependsOn(jvmCommonMain)
            dependencies {
                implementation(libs.androidx.sqlite.bundled)
            }
        }
        val nativeMain by creating { dependsOn(commonMain.get()) }
        val appleMain by creating { dependsOn(nativeMain) }
        val iosMain by creating { dependsOn(appleMain) }
        val tvosMain by creating { dependsOn(appleMain) }
        getByName("iosArm64Main").dependsOn(iosMain)
        getByName("iosSimulatorArm64Main").dependsOn(iosMain)
        getByName("tvosArm64Main").dependsOn(tvosMain)
        getByName("tvosSimulatorArm64Main").dependsOn(tvosMain)
    }
}

dependencies {
    add("kspAndroid", libs.androidx.hilt.compiler)
    add("kspAndroid", libs.androidx.room.compiler)
    add("kspDesktop", libs.androidx.room.compiler)
    add("kspIosSimulatorArm64", libs.androidx.room.compiler)
    add("kspIosArm64", libs.androidx.room.compiler)
    add("kspTvosArm64", libs.androidx.room.compiler)
    add("kspTvosSimulatorArm64", libs.androidx.room.compiler)
}
