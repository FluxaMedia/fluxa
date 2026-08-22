package com.fluxa.app.plugins

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import java.io.File
import java.security.MessageDigest

internal fun isSecureCloudStreamPluginUrl(url: String): Boolean =
    FluxaCoreNative.pluginIsSecureRemoteUrl(url)

internal fun expandCloudStreamRepositoryShortcode(input: String): String {
    val trimmed = input.trim()
    if (trimmed.contains("://")) return trimmed
    if (trimmed.startsWith("github.com/") || trimmed.startsWith("www.github.com/")) {
        return "https://$trimmed"
    }
    if (trimmed.matches(Regex("^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$"))) {
        return "https://raw.githubusercontent.com/$trimmed/builds"
    }
    return trimmed
}

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
    val expected = expectedSha256?.trim()?.lowercase()?.takeIf { it.isNotBlank() }
        ?: return "Plugin checksum is required"
    if (!expected.matches(Regex("^[a-f0-9]{64}$"))) return "Invalid plugin checksum"
    return if (sha256(file) == expected) null else "Plugin checksum verification failed"
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
