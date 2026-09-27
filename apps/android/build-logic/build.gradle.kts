plugins {
    `kotlin-dsl`
}

group = "com.fluxa.buildlogic"

dependencies {
    compileOnly(libs.android.gradlePlugin)
    compileOnly(libs.kotlin.gradlePlugin)
}

gradlePlugin {
    plugins {
        register("fluxaAndroidApplication") {
            id = "fluxa.android.application"
            implementationClass = "FluxaAndroidApplicationPlugin"
        }
        register("fluxaAndroidRust") {
            id = "fluxa.android.rust"
            implementationClass = "FluxaAndroidRustPlugin"
        }
    }
}
