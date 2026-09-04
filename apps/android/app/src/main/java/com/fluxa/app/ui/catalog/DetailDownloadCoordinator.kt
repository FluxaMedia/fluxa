package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaHeadlessAppRuntime
import com.fluxa.app.core.fromStateList
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.domain.discovery.supportsStremioResource
import com.fluxa.app.player.STREAM_SOURCE_MODE_FIRST
import com.fluxa.app.player.STREAM_SOURCE_MODE_MANUAL
import com.fluxa.app.ui.toMeta
import com.google.gson.Gson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext

internal class DetailDownloadCoordinator(
    private val runtime: FluxaHeadlessAppRuntime,
    private val gson: Gson,
    private val currentDetail: () -> MetaDetail?,
    private val seasonEpisodes: () -> List<Video>,
    private val userAddons: () -> List<AddonDescriptor>,
    private val currentProfile: () -> UserProfile?,
    private val buildRequestIds: (String, String, String) -> List<String>,
    private val fetchSubtitles: suspend (String, String, String, String) -> List<SubtitleData>
) {
    suspend fun queueEpisodes(episodes: List<Video?>): Int {
        val detail = currentDetail() ?: return 0
        val profile = currentProfile()
        return episodes
            .filterNotNull()
            .filterNot { detailIsUpcoming(it.released) }
            .count { episode -> enqueueEpisodeDownload(detail, episode, profile) }
    }

    private suspend fun enqueueEpisodeDownload(
        detail: MetaDetail,
        episode: Video,
        profile: UserProfile?
    ): Boolean {
        val requestId = episode.id.takeIf { it.isNotBlank() } ?: return false
        val language = profile?.safeLanguage ?: "en"
        val streams = fetchStreamsForDownload(detail.type, requestId, language)
        if (streams.isEmpty()) return false
        val mode = profile?.safeDownloadSourceSelectionMode ?: STREAM_SOURCE_MODE_FIRST
        val effectiveMode = if (mode == STREAM_SOURCE_MODE_MANUAL) STREAM_SOURCE_MODE_FIRST else mode
        val selectedIndex = selectStreamIndex(
            streams = streams,
            currentVideoId = requestId,
            initialStreamIndex = 0,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = effectiveMode,
            regexPattern = profile?.safeDownloadSourceRegexPattern,
            preferredBingeGroup = null
        ).takeIf { it in streams.indices } ?: 0
        val stream = streams.getOrNull(selectedIndex) ?: return false
        val subtitle = selectDownloadSubtitle(profile, detail.type, requestId, stream)
        val result = runtime.dispatch(
            mapOf(
                "type" to "offlineDownloadRequested",
                "meta" to detail.toMeta(),
                "video" to episode,
                "videoId" to requestId,
                "stream" to stream,
                "subtitle" to subtitle,
                "profileId" to profile?.id,
                "language" to language
            )
        )
        val offline = result.state["offline"] as? Map<*, *>
        return offline?.get("error") == null
    }

    private suspend fun fetchStreamsForDownload(type: String, id: String, language: String): List<Stream> {
        val result = runtime.dispatch(
            mapOf(
                "type" to "detailStreamsRequested",
                "contentType" to type,
                "requestIds" to buildRequestIds(type, id, language),
                "detail" to currentDetail(),
                "seasonEpisodes" to seasonEpisodes(),
                "language" to language,
                "profile" to currentProfile()
            )
        )
        val detail = result.state["detail"] as? Map<*, *>
        return withContext(Dispatchers.Default) {
            gson.fromStateList(detail?.get("streams"))
        }
    }

    private suspend fun selectDownloadSubtitle(
        profile: UserProfile?,
        type: String,
        id: String,
        stream: Stream
    ): OfflineSubtitleOption? {
        val setting = profile?.safeDownloadSubtitleLanguage ?: "preferred"
        if (setting == "off") return null
        val preferred = if (setting == "preferred") {
            profile?.safePreferredSubtitleLanguage
        } else {
            setting
        }?.substringBefore('-')?.substringBefore('_')?.lowercase(java.util.Locale.ROOT)
        val options = downloadSubtitleOptionsForStream(type, id, stream)
        return if (preferred.isNullOrBlank()) {
            options.firstOrNull()
        } else {
            options.firstOrNull {
                it.language?.substringBefore('-')
                    ?.substringBefore('_')
                    ?.lowercase(java.util.Locale.ROOT) == preferred
            } ?: options.firstOrNull()
        }
    }

    private suspend fun downloadSubtitleOptionsForStream(
        type: String,
        id: String,
        stream: Stream
    ): List<OfflineSubtitleOption> {
        val inline = stream.subtitles.orEmpty().mapNotNull { subtitle ->
            val url = subtitle.subtitleUrl() ?: return@mapNotNull null
            val language = subtitle.subtitleLanguages().firstOrNull()?.lowercase(java.util.Locale.ROOT)
            OfflineSubtitleOption(
                label = listOfNotNull(language, stream.addonName).joinToString(" - ").ifBlank { url },
                language = language,
                url = url
            )
        }
        val remote = withContext(Dispatchers.IO) {
            supervisorScope {
                userAddons()
                    .filter { it.supportsStremioResource("subtitles", type, id) }
                    .map { addon ->
                        async {
                            fetchSubtitles(addon.transportUrl, type, id, stream.subtitleExtraArgs())
                                .mapNotNull { subtitle ->
                                    val url = subtitle.subtitleUrl() ?: return@mapNotNull null
                                    val language = subtitle.subtitleLanguages()
                                        .firstOrNull()
                                        ?.lowercase(java.util.Locale.ROOT)
                                    OfflineSubtitleOption(
                                        label = listOfNotNull(language, addon.manifest.name)
                                            .joinToString(" - ")
                                            .ifBlank { url },
                                        language = language,
                                        url = url
                                    )
                                }
                        }
                    }
                    .map { it.await() }
                    .flatten()
            }
        }
        return inline + remote
    }
}
