package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.plugins.cloudstream.ScraperStreamLink
import com.fluxa.app.plugins.cloudstream.ScraperSubtitle
import com.google.gson.Gson

private val cloudStreamMapperGson = Gson()

internal fun List<ScraperStreamLink>.toFluxaStreams(
    addonName: String,
    subtitles: List<ScraperSubtitle>,
    sortByQuality: Boolean = false
): List<Stream> {
    val order = FluxaCoreNative.cloudstreamStreamOrder(cloudStreamMapperGson.toJson(this), sortByQuality)
    val links = order.mapNotNull(this::getOrNull)
    val subtitleData = subtitles.map(ScraperSubtitle::toSubtitleData)

    return links.map { link ->
        Stream(
            name = " $addonName\n${link.quality}",
            title = link.name,
            url = link.url,
            subtitles = subtitleData,
            behaviorHints = buildMap {
                put("proxyHeaders", buildMap { put("request", link.headers) })
                link.referer?.let { put("referer", it) }
                put("cs3Type", link.type)
                put("isM3u8", link.isM3u8)
                put("isDash", link.isDash)
            },
            addonName = " $addonName"
        )
    }
}

private fun ScraperSubtitle.toSubtitleData() = SubtitleData(
    url = url,
    lang = lang
)
