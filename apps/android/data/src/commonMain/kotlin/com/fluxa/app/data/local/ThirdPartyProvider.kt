package com.fluxa.app.data.local

/**
 * Every remote account is a peer third-party provider. A provider used for
 * authentication is not implicitly the owner of library, progress, or add-on data.
 */
enum class ThirdPartyProviderId(
    val key: String,
    val displayName: String,
    val reasonLabel: String = displayName
) {
    STREMIO("stremio", "Stremio"),
    NUVIO("nuvio", "Nuvio"),
    TRAKT("trakt", "Trakt", reasonLabel = "Trakt.tv"),
    SIMKL("simkl", "Simkl"),
    ANILIST("anilist", "AniList");

    companion object {
        fun from(value: String?): ThirdPartyProviderId? {
            val normalized = value?.trim()?.takeIf(String::isNotBlank) ?: return null
            return entries.firstOrNull {
                it.key.equals(normalized, ignoreCase = true) ||
                    it.displayName.equals(normalized, ignoreCase = true) ||
                    it.reasonLabel.equals(normalized, ignoreCase = true)
            }
        }
    }
}

data class ProviderDataOwner(
    val appProfileId: String,
    val providerId: ThirdPartyProviderId,
    val providerAccountId: String
) {
    init {
        require(appProfileId.isNotBlank())
        require(providerAccountId.isNotBlank())
    }

    /** Stored in the existing provider column so account isolation needs no DB rewrite. */
    val storageNamespace: String
        get() = "${providerId.key}::${providerAccountId.normalizedNamespacePart()}"
}

fun UserProfile.providerAccountId(providerId: ThirdPartyProviderId): String? {
    fun tokenFallback(token: String?): String? = token
        ?.takeIf(String::isNotBlank)
        ?.stableNamespaceHash()
        ?.let { "token-$it" }

    return when (providerId) {
        ThirdPartyProviderId.STREMIO -> stremioUserId?.takeIf(String::isNotBlank)
            ?: stremioEmail?.takeIf(String::isNotBlank)
            // Legacy profiles used the Fluxa profile email as their Stremio identity.
            ?: email.takeIf { authKey.isNotBlank() && it.isNotBlank() }
            ?: tokenFallback(authKey)
        ThirdPartyProviderId.NUVIO -> {
            val account = nuvioUserId?.takeIf(String::isNotBlank)
                ?: nuvioEmail?.takeIf(String::isNotBlank)
                ?: tokenFallback(nuvioAccessToken)
            account?.let { "$it#${nuvioProfileIndex ?: 1}" }
        }
        ThirdPartyProviderId.TRAKT -> traktUsername?.takeIf(String::isNotBlank)
            ?: tokenFallback(traktRefreshToken)
            ?: tokenFallback(traktAccessToken)
        ThirdPartyProviderId.SIMKL -> simklUsername?.takeIf(String::isNotBlank)
            ?: tokenFallback(simklAccessToken)
        ThirdPartyProviderId.ANILIST -> anilistUsername?.takeIf(String::isNotBlank)
            ?: tokenFallback(anilistAccessToken)
    }
}

fun UserProfile.providerDataOwner(providerId: ThirdPartyProviderId): ProviderDataOwner? =
    providerAccountId(providerId)?.let { ProviderDataOwner(id, providerId, it) }

fun UserProfile.providerLastSyncAt(providerId: ThirdPartyProviderId): Long =
    providerSyncTimestamps?.get(providerId.key)
        ?: when (providerId) {
            ThirdPartyProviderId.NUVIO -> nuvioLastSyncAt
            ThirdPartyProviderId.TRAKT -> traktLastSyncAt
            ThirdPartyProviderId.SIMKL -> simklLastSyncAt
            ThirdPartyProviderId.STREMIO, ThirdPartyProviderId.ANILIST -> null
        }
        ?: 0L

fun UserProfile.withProviderLastSyncAt(
    providerId: ThirdPartyProviderId,
    timestamp: Long
): UserProfile = copy(
    providerSyncTimestamps = providerSyncTimestamps.orEmpty() + (providerId.key to timestamp),
    nuvioLastSyncAt = if (providerId == ThirdPartyProviderId.NUVIO) timestamp else nuvioLastSyncAt,
    traktLastSyncAt = if (providerId == ThirdPartyProviderId.TRAKT) timestamp else traktLastSyncAt,
    simklLastSyncAt = if (providerId == ThirdPartyProviderId.SIMKL) timestamp else simklLastSyncAt
)

fun UserProfile.isProviderConnected(providerId: ThirdPartyProviderId): Boolean = when (providerId) {
    ThirdPartyProviderId.STREMIO -> authKey.isNotBlank()
    ThirdPartyProviderId.NUVIO -> !nuvioAccessToken.isNullOrBlank()
    ThirdPartyProviderId.TRAKT -> !traktAccessToken.isNullOrBlank()
    ThirdPartyProviderId.SIMKL -> !simklAccessToken.isNullOrBlank()
    ThirdPartyProviderId.ANILIST -> !anilistAccessToken.isNullOrBlank()
}

fun String.isThirdPartyProviderReason(): Boolean =
    ThirdPartyProviderId.from(this) != null

private fun String.normalizedNamespacePart(): String = trim()
    .lowercase()
    .replace("::", "_")
    .take(180)

/** Stable non-reversible identifier for accounts whose backend exposes no user id. */
private fun String.stableNamespaceHash(): String {
    var hash = 0xcbf29ce484222325UL
    encodeToByteArray().forEach { byte ->
        hash = hash xor byte.toUByte().toULong()
        hash *= 0x100000001b3UL
    }
    return hash.toString(16).padStart(16, '0')
}
