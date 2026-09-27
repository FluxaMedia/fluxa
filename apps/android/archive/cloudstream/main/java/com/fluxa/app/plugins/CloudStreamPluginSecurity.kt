package com.fluxa.app.plugins

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import java.io.File
import java.security.MessageDigest

internal fun isSecureCloudStreamPluginUrl(url: String): Boolean =
    FluxaCoreNative.pluginIsSecureRemoteUrl(url)

internal fun expandCloudStreamRepositoryShortcode(input: String): String =
    FluxaCoreNative.normalizeCloudstreamRepoInput(input)

internal fun normalizeCloudStreamRepositoryUrl(url: String): String =
    FluxaCoreNative.normalizePluginRepositoryUrl(url)

internal fun sameCloudStreamRepositoryUrl(left: String, right: String): Boolean =
    FluxaCoreNative.pluginSameRepositoryUrl(left, right)

internal fun cloudStreamPluginInstallId(repositoryUrl: String?, internalName: String): String {
    val scope = repositoryUrl?.takeIf { it.isNotBlank() } ?: "manual"
    return PluginStateCodec.sha256("$scope:$internalName").take(24)
}

internal fun InstalledPlugin.cloudStreamInstallKey(): String =
    installId ?: cloudStreamPluginInstallId(repositoryUrl, internalName)

internal fun verifyCloudStreamPluginChecksum(file: File, expectedSha256: String?): String? {
    return when (FluxaCoreNative.sha256VerificationStatus(expectedSha256, sha256(file))) {
        "ok" -> null
        "missing" -> "Plugin checksum is required"
        "invalid" -> "Invalid plugin checksum"
        else -> "Plugin checksum verification failed"
    }
}

internal fun sha256(file: File): String {
    val digest = MessageDigest.getInstance("SHA-256")
    file.inputStream().use { input ->
        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
        while (true) {
            val read = input.read(buffer)
            if (read <= 0) break
            digest.update(buffer, 0, read)
        }
    }
    return digest.digest().joinToString("") { "%02x".format(it) }
}
