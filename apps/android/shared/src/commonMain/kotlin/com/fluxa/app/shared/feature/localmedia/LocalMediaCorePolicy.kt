package com.fluxa.app.shared.feature.localmedia

/** Platform-neutral local-media policy backed by fluxa-core through [LocalMediaCorePort]. */
internal interface LocalMediaCorePort {
    fun parseFilename(fileName: String, parentHints: List<String>, kind: LocalMediaKind): LocalMediaParsedName?
    fun scoreCandidate(parsed: LocalMediaParsedName, meta: LocalMediaCoreMeta, kind: LocalMediaKind): Float
    fun resolveVideo(parsed: LocalMediaParsedName, videos: List<LocalMediaCoreVideo>): LocalMediaCoreVideo?
    fun normalizedTitle(value: String): String
}

internal class LocalMediaCorePolicy(
    private val port: LocalMediaCorePort,
) {
    fun parse(fileName: String, parentHints: List<String>, kind: LocalMediaKind): LocalMediaParsedName? =
        port.parseFilename(fileName, parentHints, kind)

    fun score(parsed: LocalMediaParsedName, meta: LocalMediaCoreMeta, kind: LocalMediaKind): Float =
        port.scoreCandidate(parsed, meta, kind)

    fun resolveVideo(parsed: LocalMediaParsedName, videos: List<LocalMediaCoreVideo>): LocalMediaCoreVideo? =
        port.resolveVideo(parsed, videos)

    fun normalizedTitle(value: String): String = port.normalizedTitle(value)
}
