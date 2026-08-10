package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.NuvioLibraryItemDto
import com.fluxa.app.data.remote.NuvioWatchProgressDto

/**
 * Canonical Nuvio progress identity. The server normally supplies `progress_key`,
 * but the public v1.2 contract guarantees that it is derived from
 * `content_id + season + episode`, so older rows can be repaired client-side.
 */
internal fun NuvioWatchProgressDto.canonicalProgressKey(): String {
    progressKey?.trim()?.takeIf(String::isNotBlank)?.let { return it }
    val content = contentId.trim()
    return if (season != null && episode != null) {
        "${content}_s${season}e${episode}"
    } else {
        content
    }
}

/**
 * Maps Nuvio-owned playback progress. Library metadata remains authoritative;
 * optional metadata detail is only a fallback resolved from this Nuvio
 * account's own enabled add-ons.
 */
internal fun NuvioWatchProgressDto.toContinueWatchingMeta(
    libraryItem: NuvioLibraryItemDto?,
    metadataDetail: MetaDetail? = null,
): Meta? {
    if (contentId.isBlank() || duration <= 0L || position < 0L) return null

    val item = libraryItem?.toDomain()
    val detail = metadataDetail
    val resolvedName = item?.name
        ?.trim()
        ?.takeIf { it.isNotBlank() && !it.equals(contentId, ignoreCase = true) }
        ?: detail?.name?.trim()?.takeIf(String::isNotBlank)
        ?: contentId
    val resolvedType = item?.contentType?.takeIf(String::isNotBlank)
        ?: detail?.type?.takeIf(String::isNotBlank)
        ?: contentType
    val resolvedPoster = item?.poster?.takeIf(String::isNotBlank)
        ?: detail?.poster?.takeIf(String::isNotBlank)
    val resolvedBackground = item?.background?.takeIf(String::isNotBlank)
        ?: detail?.background?.takeIf(String::isNotBlank)
    val episodeVideo = detail?.videos
        ?.firstOrNull { video -> video.id == videoId }
        ?: detail?.videos?.firstOrNull { video ->
            season != null && episode != null && video.season == season && video.number == episode
        }
    val episodeStill = episodeVideo?.thumbnail?.takeIf(String::isNotBlank)
    val episodeTitle = episodeVideo?.name?.trim()?.takeIf(String::isNotBlank)
    val episodeCode = if (season != null && episode != null) "S$season E$episode" else null

    return Meta(
        id = contentId,
        name = resolvedName,
        type = resolvedType,
        poster = resolvedPoster,
        background = resolvedBackground,
        logo = detail?.logo,
        description = item?.description?.takeIf(String::isNotBlank) ?: detail?.description,
        releaseInfo = item?.releaseInfo?.takeIf(String::isNotBlank) ?: detail?.releaseInfo,
        released = detail?.released,
        runtime = detail?.runtime,
        imdbRating = item?.imdbRating?.toString() ?: detail?.imdbRating,
        ageRating = detail?.ageRating,
        genres = item?.genres?.takeIf { it.isNotEmpty() } ?: detail?.genres,
        seasonsCount = detail?.seasonsCount,
        timeOffset = position,
        duration = duration,
        resumeProgressPercent = ((position.toDouble() / duration.toDouble()) * 100.0)
            .toFloat()
            .coerceIn(0f, 100f),
        lastVideoId = videoId,
        // Preserve the season/episode code in provider data as well as the real title. Some
        // Nuvio video ids do not encode S/E, so the UI cannot always recover the code from id alone.
        lastEpisodeName = when {
            episodeCode != null && episodeTitle != null -> "$episodeCode · $episodeTitle"
            episodeTitle != null -> episodeTitle
            else -> episodeCode
        },
        lastWatchedAt = lastWatched,
        reason = "Nuvio",
        continueWatchingPoster = resolvedPoster,
        continueWatchingBackground = episodeStill ?: resolvedBackground ?: resolvedPoster,
    )
}
