package com.fluxa.app.shared.feature.localmedia

import kotlin.math.max

/** Pure parser shared by every platform and every filesystem backend. */
object LocalMediaFilenameParser {
    private val videoExtensions = setOf("mkv", "mp4", "m4v", "avi", "mov", "webm", "ts", "m2ts", "wmv", "flv")
    private val seasonEpisode = Regex("(?i)(?:^|[ ._\\-])S(\\d{1,3})[ ._\\-]*E(\\d{1,4})(?:[^0-9]|$)")
    private val seasonEpisodeAlt = Regex("(?i)(?:^|[ ._\\-])(\\d{1,2})x(\\d{1,4})(?:[^0-9]|$)")
    private val episodeOnly = Regex("(?i)(?:^|[ ._\\-])(?:EP?|Episode)[ ._\\-]*(\\d{1,4})(?:[^0-9]|$)")
    private val animeAbsolute = Regex("(?:^|[ ._\\-])-?[ ._]*(\\d{1,4})(?:v\\d+)?(?=[ ._\\-]*(?:\\[|\\(|$))", RegexOption.IGNORE_CASE)
    private val yearPattern = Regex("(?:^|[^0-9])((?:19|20)\\d{2})(?:[^0-9]|$)")
    private val explicitId = Regex("(?i)(?:\\{|\\[|\\()?(imdb|tmdb)[-_: ](tt\\d+|\\d+)(?:\\}|\\]|\\))?")
    private val releaseNoise = Regex(
        "(?i)\\b(?:2160p|1080p|720p|480p|uhd|bluray|blu-ray|bdrip|brrip|web[- .]?dl|webrip|hdtv|remux|x26[45]|h26[45]|hevc|av1|hdr10\\+?|hdr|dv|dolby[ .]?vision|atmos|truehd|dts(?:-hd)?|aac|ddp?\\d?(?:\\.\\d)?|proper|repack|extended|multi|dual|nf|amzn|dsnp|hmax)\\b"
    )
    private val bracketGroup = Regex("^\\[[^]]+][ ._-]*")

    data class ParsedName(
        val title: String,
        val year: Int?,
        val season: Int?,
        val episode: Int?,
        val absoluteEpisode: Int?,
        val explicitMetadataId: String? = null,
        val explicitMetadataProvider: String? = null,
    )

    fun isVideoFile(name: String): Boolean = name.substringAfterLast('.', "").lowercase() in videoExtensions

    fun parse(fileName: String, parentHints: List<String>, kind: LocalMediaKind): ParsedName? {
        if (!isVideoFile(fileName)) return null
        val stem = fileName.substringBeforeLast('.')
        val explicit = explicitId.find(stem) ?: parentHints.firstNotNullOfOrNull { explicitId.find(it) }
        val seasonMatch = seasonEpisode.find(stem) ?: seasonEpisodeAlt.find(stem)
        val season = seasonMatch?.groupValues?.getOrNull(1)?.toIntOrNull()
        val episode = seasonMatch?.groupValues?.getOrNull(2)?.toIntOrNull()
        val episodeOnlyValue = if (seasonMatch == null) episodeOnly.find(stem)?.groupValues?.getOrNull(1)?.toIntOrNull() else null
        val absolute = if (kind == LocalMediaKind.Anime && seasonMatch == null) {
            episodeOnlyValue ?: animeAbsolute.find(stem)?.groupValues?.getOrNull(1)?.toIntOrNull()
        } else null
        val year = yearPattern.find(stem)?.groupValues?.getOrNull(1)?.toIntOrNull()
            ?: parentHints.firstNotNullOfOrNull { yearPattern.find(it)?.groupValues?.getOrNull(1)?.toIntOrNull() }
        val genericFolder = Regex("(?i)^(?:movies?|films?|tv(?:[ ._-]*shows?)?|series|shows?|anime|media|season[ ._-]*\\d+|s\\d+)$")
        val parentTitle = if (kind == LocalMediaKind.Movies) null else parentHints.firstOrNull { hint ->
            hint.isNotBlank() && !genericFolder.matches(hint.trim())
        }
        val titleSource = parentTitle ?: stem
        val title = cleanTitle(
            titleSource,
            year,
            episodeStart = if (parentTitle == null) seasonMatch?.range?.first else null,
        )
        if (title.isBlank()) return null
        return ParsedName(
            title = title,
            year = year,
            season = season,
            episode = episode ?: if (kind == LocalMediaKind.TvShows) episodeOnlyValue else null,
            absoluteEpisode = absolute,
            explicitMetadataId = explicit?.groupValues?.getOrNull(2)?.takeIf(String::isNotBlank),
            explicitMetadataProvider = explicit?.groupValues?.getOrNull(1)?.lowercase()?.takeIf(String::isNotBlank),
        )
    }

    fun normalizedTitle(value: String): String = value
        .lowercase()
        .replace('&', ' ')
        .replace(Regex("[^a-z0-9]+"), " ")
        .trim()
        .replace(Regex("\\s+"), " ")

    fun titleSimilarity(a: String, b: String): Float {
        val x = normalizedTitle(a)
        val y = normalizedTitle(b)
        if (x == y) return 1f
        if (x.isBlank() || y.isBlank()) return 0f
        val xa = x.split(' ').filter(String::isNotBlank).toSet()
        val ya = y.split(' ').filter(String::isNotBlank).toSet()
        val union = xa union ya
        val tokenScore = if (union.isEmpty()) 0f else (xa intersect ya).size.toFloat() / union.size
        val prefixScore = commonPrefix(x, y).toFloat() / max(x.length, y.length).coerceAtLeast(1)
        return (tokenScore * 0.82f + prefixScore * 0.18f).coerceIn(0f, 1f)
    }

    private fun commonPrefix(a: String, b: String): Int {
        val limit = minOf(a.length, b.length)
        var i = 0
        while (i < limit && a[i] == b[i]) i++
        return i
    }

    private fun cleanTitle(raw: String, year: Int?, episodeStart: Int?): String {
        var value = raw
        if (episodeStart != null && episodeStart in 1 until value.length) value = value.substring(0, episodeStart)
        value = bracketGroup.replace(value, "")
        explicitId.findAll(value).forEach { value = value.replace(it.value, " ") }
        if (year != null) value = value.replace(year.toString(), " ")
        value = releaseNoise.replace(value, " ")
        value = value
            .replace(Regex("(?i)[ ._\\-]+S\\d{1,3}[ ._\\-]*E\\d{1,4}.*$"), "")
            .replace(Regex("(?i)[ ._\\-]+\\d{1,2}x\\d{1,4}.*$"), "")
            .replace(Regex("[._]+"), " ")
            .replace(Regex("\\s+-\\s+\\d{1,4}.*$"), "")
            .replace(Regex("\\s+"), " ")
            .trim(' ', '-', '_', '.')
        return value
    }
}
