import com.android.build.gradle.LibraryExtension
import com.android.build.gradle.internal.dsl.BaseAppModuleExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.compose.compiler.gradle.ComposeCompilerGradlePluginExtension

class FluxaAndroidComposePlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.pluginManager.apply("org.jetbrains.kotlin.plugin.compose")

        target.extensions.configure<ComposeCompilerGradlePluginExtension> {
            val conf = target.layout.projectDirectory.file("compose-stability.conf")
            if (conf.asFile.exists()) {
                stabilityConfigurationFiles.add(conf)
            }
        }

        target.pluginManager.withPlugin("com.android.application") {
            target.extensions.configure<BaseAppModuleExtension> {
                buildFeatures { compose = true }
            }
        }
        target.pluginManager.withPlugin("com.android.library") {
            target.extensions.configure<LibraryExtension> {
                buildFeatures { compose = true }
            }
        }
    }
}
