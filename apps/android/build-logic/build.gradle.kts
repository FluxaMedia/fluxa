plugins {
    `kotlin-dsl`
}

group = "com.fluxa.buildlogic"

dependencies {
    compileOnly(libs.android.gradlePlugin)
    compileOnly(libs.kotlin.gradlePlugin)
    compileOnly(libs.kotlin.composeCompilerGradlePlugin)
    compileOnly(libs.ksp.gradlePlugin)
    compileOnly(libs.compose.multiplatform.gradlePlugin)
}

gradlePlugin {
    plugins {
        register("fluxaAndroidLibrary") {
            id = "fluxa.android.library"
            implementationClass = "FluxaAndroidLibraryPlugin"
        }
        register("fluxaAndroidApplication") {
            id = "fluxa.android.application"
            implementationClass = "FluxaAndroidApplicationPlugin"
        }
        register("fluxaAndroidCompose") {
            id = "fluxa.android.compose"
            implementationClass = "FluxaAndroidComposePlugin"
        }
        register("fluxaAndroidHilt") {
            id = "fluxa.android.hilt"
            implementationClass = "FluxaAndroidHiltPlugin"
        }
        register("fluxaKmpLibrary") {
            id = "fluxa.kmp.library"
            implementationClass = "FluxaKmpLibraryPlugin"
        }
        register("fluxaKmpCompose") {
            id = "fluxa.kmp.compose"
            implementationClass = "FluxaKmpComposePlugin"
        }
    }
}
