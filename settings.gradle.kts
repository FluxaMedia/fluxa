pluginManagement {
    includeBuild("build-logic")
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven { url = uri("https://jitpack.io") }
        maven { url = uri("https://jogamp.org/deployment/maven") }
    }
}

rootProject.name = "Fluxa"
include(":core")
include(":shared")
include(":data")
include(":player")
include(":app")
include(":tvBenchmark")
include(":mobileBenchmark")
