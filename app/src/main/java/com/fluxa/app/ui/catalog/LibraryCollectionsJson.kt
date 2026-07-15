package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder

internal fun LibraryUserCollectionFolder.effectiveImageUrl(): String? {
    return coverImageUrl.cleanedArtworkUrl() ?: imageUrl.cleanedArtworkUrl()
}

internal fun LibraryUserCollectionFolder.effectiveShape(): String {
    return shape ?: normalizeImportedTileShape(null)
}

internal fun LibraryUserCollection.effectiveFolderShape(folder: LibraryUserCollectionFolder): String? {
    return folder.effectiveShape()
}

private fun normalizeImportedTileShape(value: String?): String {
    return when (value?.trim()?.uppercase()) {
        "LANDSCAPE", "WIDE" -> "wide"
        "SQUARE" -> "square"
        else -> "poster"
    }
}

private fun String?.cleanedUrl(): String? {
    return this?.trim()?.takeIf { it.isNotBlank() }
}

private fun String?.cleanedArtworkUrl(): String? {
    val raw = cleanedUrl()
        ?.trim('"', '\'')
        ?.takeIf { it.isNotBlank() }
        ?: return null
    val withScheme = when {
        raw.startsWith("//") -> "https:$raw"
        else -> raw
    }
    val githubBlob = Regex("""^https://github\.com/([^/]+)/([^/]+)/blob/([^/]+)/(.+)$""")
    val normalized = githubBlob.matchEntire(withScheme)?.let { match ->
        val (owner, repo, branch, path) = match.destructured
        "https://raw.githubusercontent.com/$owner/$repo/$branch/$path"
    } ?: withScheme
    return normalized.replace(" ", "%20")
}
