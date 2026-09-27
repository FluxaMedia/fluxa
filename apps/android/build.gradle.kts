import org.gradle.api.GradleException

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.test) apply false
    alias(libs.plugins.kotlin.android) apply false
}

val maxKotlinFileLines = 1500

tasks.register("checkKotlinFileSize") {
    group = "verification"
    description = "Fails when Kotlin source files exceed the local maintainability budget."

    doLast {
        val oversizedFiles = fileTree(rootDir) {
            include("app/src/main/java/**/*.kt")
            exclude("**/build/**")
        }.files.mapNotNull { file ->
            val lines = file.readLines().size
            if (lines > maxKotlinFileLines) "${file.relativeTo(rootDir)}: $lines" else null
        }

        if (oversizedFiles.isNotEmpty()) {
            throw GradleException(
                "Kotlin files exceed $maxKotlinFileLines lines:\n${oversizedFiles.joinToString("\n")}"
            )
        }
    }
}

tasks.register("qualityCheck") {
    group = "verification"
    description = "Runs the default local quality gate for Fluxa."
    dependsOn(
        "checkKotlinFileSize",
        ":app:assembleMobileDebug",
        ":app:assembleTvDebug"
    )
}
