package com.fluxa.app.domain.playback

import com.fluxa.app.data.remote.Meta

data class TraktScrobbleSchedule(
    val profileId: String,
    val providerAccountId: String,
    val mediaType: String,
    val mediaId: String,
    val progress: Float,
    val action: String,
)

data class SimklScrobbleSchedule(
    val profileId: String,
    val providerAccountId: String,
    val mediaType: String,
    val mediaId: String,
    val action: String,
    val positionMs: Long,
    val durationMs: Long,
)

data class StremioPlaybackProgressSchedule(
    val profileId: String,
    val providerAccountId: String,
    val meta: Meta,
    val timeOffset: Long,
    val duration: Long,
)

data class NuvioPlaybackProgressSchedule(
    val profileId: String,
    val providerAccountId: String,
    val contentId: String,
    val contentType: String,
    val videoId: String?,
    val position: Long,
    val duration: Long,
)

interface PlaybackProgressScheduler {
    fun scheduleTraktScrobble(request: TraktScrobbleSchedule): Boolean
    fun scheduleSimklScrobble(request: SimklScrobbleSchedule): Boolean
    fun scheduleStremioProgress(request: StremioPlaybackProgressSchedule): Boolean
    fun scheduleNuvioProgress(request: NuvioPlaybackProgressSchedule): Boolean
}
