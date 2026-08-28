import com.android.build.gradle.internal.dsl.BaseAppModuleExtension
import org.gradle.api.GradleException
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.testing.Test
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.register
import org.gradle.kotlin.dsl.withType
import java.io.File

class FluxaAndroidRustPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
        val rustCoreDir = rootProject.layout.projectDirectory.asFile
            .resolve("../../core/fluxa-core")
            .canonicalFile
        val streamingDir = rustCoreDir.resolve("fluxa-streaming-engine")
        val outputDir = layout.buildDirectory.dir("generated/rustJniLibs")
        val targets = listOf(
            RustTarget("arm64-v8a", "aarch64-linux-android", "AARCH64_LINUX_ANDROID"),
            RustTarget("armeabi-v7a", "armv7-linux-androideabi", "ARMV7_LINUX_ANDROIDEABI"),
            RustTarget("x86", "i686-linux-android", "I686_LINUX_ANDROID"),
        )
        val releaseBuild = gradle.startParameter.taskNames.any {
            it.contains("release", ignoreCase = true) || it.contains("benchmark", ignoreCase = true)
        }
        val profile = if (releaseBuild) "release" else "debug"
        val cargoProfileArgs = if (releaseBuild) listOf("--release") else emptyList()
        val allAbis = targets.map { it.abi }
        val selectedAbis = providers.gradleProperty("fluxaRustAbis").orNull
            ?.split(',')
            ?.map(String::trim)
            ?.filter(String::isNotEmpty)
            ?.distinct()
            ?: if (releaseBuild) allAbis else listOf("arm64-v8a")
        val unknownAbis = selectedAbis - allAbis.toSet()
        if (unknownAbis.isNotEmpty()) {
            throw GradleException(
                "Unknown fluxaRustAbis: ${unknownAbis.joinToString()}. " +
                    "Supported values: ${allAbis.joinToString()}"
            )
        }
        if (selectedAbis.isEmpty()) {
            throw GradleException("fluxaRustAbis must contain at least one ABI.")
        }
        val selectedTargets = targets.filter { it.abi in selectedAbis }
        val hostTag = when {
            org.gradle.internal.os.OperatingSystem.current().isMacOsX -> "darwin-x86_64"
            org.gradle.internal.os.OperatingSystem.current().isWindows -> "windows-x86_64"
            else -> "linux-x86_64"
        }
        val ndkDir = providers.environmentVariable("ANDROID_NDK_HOME").orElse(
            providers.environmentVariable("ANDROID_NDK_ROOT").orElse(
                providers.provider {
                    val sdkDir = rootProject.file("local.properties")
                        .takeIf(File::exists)
                        ?.readLines()
                        ?.firstOrNull { it.startsWith("sdk.dir=") }
                        ?.substringAfter("sdk.dir=")
                    listOfNotNull(
                        sdkDir?.let { "$it/ndk/28.2.13676358" },
                        "${System.getProperty("user.home")}/Android/Sdk/ndk/28.2.13676358",
                        "${System.getProperty("user.home")}/Android/Sdk/ndk/27.1.12297006",
                    ).firstOrNull { file(it).exists() }.orEmpty()
                }
            )
        )

        extensions.configure<BaseAppModuleExtension> {
            sourceSets.getByName("main").jniLibs.srcDir(outputDir)
        }

        fun configureToolchain(task: Exec, target: RustTarget) {
            task.doFirst {
                val ndk = ndkDir.get()
                if (ndk.isBlank() || !file(ndk).exists()) {
                    throw GradleException("Android NDK is required to build Fluxa Rust libraries.")
                }
                val toolchainRoot = file("$ndk/toolchains/llvm/prebuilt/$hostTag")
                val toolchainBin = toolchainRoot.resolve("bin")
                val linkerName = target.linkerName
                val clang = toolchainBin.resolve(linkerName).absolutePath
                task.environment("CARGO_TARGET_${target.envName}_LINKER", clang)
                task.environment("CC_${target.triple}", clang)
                task.environment("CC_${target.triple.replace('-', '_')}", clang)
                task.environment("AR_${target.triple}", toolchainBin.resolve("llvm-ar").absolutePath)
                task.environment("AR_${target.triple.replace('-', '_')}", toolchainBin.resolve("llvm-ar").absolutePath)
                val clangVersionDir = toolchainRoot.resolve("lib/clang").listFiles()
                    ?.filter(File::isDirectory)
                    ?.maxByOrNull { it.name.toIntOrNull() ?: -1 }
                task.environment("LIBCLANG_PATH", toolchainRoot.resolve("lib").absolutePath)
                task.environment(
                    "BINDGEN_EXTRA_CLANG_ARGS",
                    buildString {
                        append("--target=${linkerName.removeSuffix("-clang")}")
                        append(" --sysroot=${toolchainRoot.resolve("sysroot").absolutePath}")
                        if (clangVersionDir != null) append(" -resource-dir=${clangVersionDir.absolutePath}")
                    },
                )
            }
        }

        val coreTasks = selectedTargets.map { target ->
            tasks.register<Exec>("buildFluxaCore${target.taskSuffix}") {
                group = "build"
                description = "Builds the Fluxa Rust core for ${target.abi}."
                workingDir = rustCoreDir
                commandLine("cargo", "build", "--target", target.triple, *cargoProfileArgs.toTypedArray())
                inputs.files(fileTree(rustCoreDir) {
                    exclude("target/**", ".git/**", ".agents/**", ".codex/**", "fluxa-streaming-engine/target/**")
                })
                outputs.file(outputDir.map { it.file("${target.abi}/libfluxa_core.so") })
                configureToolchain(this, target)
                doLast {
                    copyLibrary(
                        rustCoreDir,
                        target,
                        profile,
                        "libfluxa_core.so",
                        outputDir.get().dir(target.abi).asFile,
                    )
                }
            }
        }
        tasks.register("buildFluxaCore") {
            group = "build"
            description = "Builds the Fluxa Rust core for selected Android ABIs."
            dependsOn(coreTasks)
        }

        val streamingTasks = selectedTargets.map { target ->
            tasks.register<Exec>("buildFluxaStreamingEngine${target.taskSuffix}") {
                group = "build"
                description = "Builds the Fluxa streaming engine for ${target.abi}."
                workingDir = streamingDir
                commandLine("cargo", "build", "--target", target.triple, *cargoProfileArgs.toTypedArray())
                inputs.files(fileTree(streamingDir) { exclude("target/**", ".git/**", ".agents/**", ".codex/**") })
                outputs.file(outputDir.map { it.file("${target.abi}/libfluxa_streaming_engine.so") })
                configureToolchain(this, target)
                doLast {
                    copyLibrary(
                        rustCoreDir,
                        target,
                        profile,
                        "libfluxa_streaming_engine.so",
                        outputDir.get().dir(target.abi).asFile,
                    )
                }
            }
        }
        tasks.register("buildFluxaStreamingEngine") {
            group = "build"
            description = "Builds the Fluxa streaming engine for selected Android ABIs."
            dependsOn(streamingTasks)
        }
        tasks.matching { it.name == "preBuild" }.configureEach {
            dependsOn("buildFluxaCore", "buildFluxaStreamingEngine")
        }
        tasks.withType<Test>().configureEach {
            dependsOn(rootProject.tasks.named("buildFluxaCoreHost"))
            dependsOn(rootProject.tasks.named("buildFluxaStreamingEngineHost"))
            jvmArgs("-Djava.library.path=${rustCoreDir.resolve("target/debug").absolutePath}")
            systemProperty("jna.library.path", rustCoreDir.resolve("target/debug").absolutePath)
            systemProperty("fluxa.core.library.path", rustCoreDir.resolve("target/debug/libfluxa_core.so").absolutePath)
        }
        }
    }

    private data class RustTarget(val abi: String, val triple: String, val envName: String) {
        val taskSuffix = abi.split('-', '_').joinToString("") { it.replaceFirstChar(Char::uppercaseChar) }
        val linkerName = if (triple == "armv7-linux-androideabi") "armv7a-linux-androideabi26-clang" else "${triple}26-clang"
    }

    private fun Project.copyLibrary(crateDir: File, target: RustTarget, profile: String, name: String, output: File) {
        val built = crateDir.resolve("target/${target.triple}/$profile/$name")
        if (!built.exists()) throw GradleException("Rust build did not produce ${built.absolutePath}")
        copy { from(built); into(output) }
    }
}
