package com.fluxa.app.data.local

data class OfflineSubtitleOption(
    val label: String,
    val language: String?,
    val url: String
)

data class OfflineDownloadItem(
    val id: String,
    val profileId: String?,
    val language: String? = null,
    val metaId: String,
    val metaType: String,
    val title: String,
    val episodeTitle: String?,
    val videoId: String?,
    val poster: String?,
    val background: String?,
    val logo: String? = null,
    val localPosterPath: String? = null,
    val localBackgroundPath: String? = null,
    val localLogoPath: String? = null,
    val streamTitle: String? = null,
    val videoPath: String = "",
    val subtitlePath: String? = null,
    val subtitleLabel: String? = null,
    val subtitleLanguage: String? = null,
    val downloadId: Long = 0L,
    val createdAt: Long = 0L,
    val status: String = "queued",
    val progress: Int = 0,
    val downloadedBytes: Long = 0L,
    val totalBytes: Long = 0L,
    val speedBytesPerSecond: Long = 0L,
    val etaSeconds: Long = -1L,
    val error: String? = null
)
