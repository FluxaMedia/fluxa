import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.kotlin.dsl.dependencies
import org.gradle.kotlin.dsl.getByType

class FluxaAndroidHiltPlugin : Plugin<Project> {
    override fun apply(target: Project) = with(target) {
        pluginManager.apply("com.google.dagger.hilt.android")
        pluginManager.apply("com.google.devtools.ksp")

        val libs = extensions.getByType<VersionCatalogsExtension>().named("libs")
        dependencies {
            "implementation"(libs.findLibrary("hilt-android").get())
            "ksp"(libs.findLibrary("hilt-compiler").get())
            "ksp"(libs.findLibrary("kotlin-metadata-jvm").get())
        }
    }
}
