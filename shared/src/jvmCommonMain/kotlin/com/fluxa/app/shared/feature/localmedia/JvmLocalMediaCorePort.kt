package com.fluxa.app.shared.feature.localmedia

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.NativeLocalMediaParsedName
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video

internal object JvmLocalMediaCorePort : LocalMediaCorePort {
    override fun parseFilename(fileName: String, parentHints: List<String>, kind: LocalMediaKind): LocalMediaParsedName? {
        val value = FluxaCoreNative.localMediaParseFilename(fileName, parentHints, kind.wireName()) ?: return null
        return value.toLocalMediaParsedName()
    }

    override fun scoreCandidate(parsed: LocalMediaParsedName, meta: LocalMediaCoreMeta, kind: LocalMediaKind): Float =
        FluxaCoreNative.localMediaScoreCandidate(parsed.toNative(), meta.toNative(), kind.wireName())

    override fun resolveVideo(parsed: LocalMediaParsedName, videos: List<LocalMediaCoreVideo>): LocalMediaCoreVideo? =
        FluxaCoreNative.localMediaResolveVideo(parsed.toNative(), videos.map { it.toNative() })?.toLocalMediaCoreVideo()

    override fun normalizedTitle(value: String): String = FluxaCoreNative.localMediaNormalizedTitle(value)

    private fun LocalMediaKind.wireName(): String = when (this) {
        LocalMediaKind.Movies -> "movies"
        LocalMediaKind.TvShows -> "tvShows"
        LocalMediaKind.Anime -> "anime"
    }

    private fun NativeLocalMediaParsedName.toLocalMediaParsedName() = LocalMediaParsedName(
        title, year, season, episode, absoluteEpisode, explicitMetadataId, explicitMetadataProvider,
    )

    private fun LocalMediaParsedName.toNative() = NativeLocalMediaParsedName(
        title, year, season, episode, absoluteEpisode, explicitMetadataId, explicitMetadataProvider,
    )

    private fun LocalMediaCoreMeta.toNative() = Meta(id = id, name = name, type = type, releaseInfo = releaseInfo, released = released)
    private fun LocalMediaCoreVideo.toNative() = Video(id = id, season = season, number = number)
    private fun Video.toLocalMediaCoreVideo() = LocalMediaCoreVideo(id, season, number)
}

internal val jvmLocalMediaCorePolicy = LocalMediaCorePolicy(JvmLocalMediaCorePort)
