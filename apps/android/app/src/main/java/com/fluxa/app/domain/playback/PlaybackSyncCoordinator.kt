package com.fluxa.app.domain.playback

import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.isProviderConnected
import com.fluxa.app.data.local.providerAccountId
import com.fluxa.app.data.local.safeContinueWatchingSource
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Playback sync fan-out plus Continue Watching source selection.
 *
 * [continueWatchingProvider], [cachedContinueWatching] and the read-side helpers are only about
 * which provider Fluxa displays in the Continue Watching row. Playback writes never use
 * that preference: every connected provider that supports the operation is a sync target.
 */
@Singleton
class PlaybackSyncCoordinator @Inject constructor(
    private val scheduler: PlaybackProgressScheduler,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository
) {
    fun continueWatchingProvider(profile: UserProfile?): ThirdPartyProviderId? =
        profile?.let { ThirdPartyProviderId.from(it.safeContinueWatchingSource) }

    fun isLocalContinueWatchingSource(profile: UserProfile?): Boolean = continueWatchingProvider(profile) == null

    fun scheduleProgress(
        profile: UserProfile,
        meta: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        allowPauseScrobble: Boolean
    ) {
        if (durationMs <= 0L) return

        if (profile.isProviderConnected(ThirdPartyProviderId.STREMIO)) {
            profile.providerAccountId(ThirdPartyProviderId.STREMIO)?.let { accountId ->
                val stremioMeta = if (!videoId.isNullOrBlank()) meta.copy(lastVideoId = videoId) else meta
                scheduler.scheduleStremioProgress(
                    StremioPlaybackProgressSchedule(profile.id, accountId, stremioMeta, positionMs, durationMs)
                )
            }
        }
        if (profile.isProviderConnected(ThirdPartyProviderId.NUVIO)) {
            profile.providerAccountId(ThirdPartyProviderId.NUVIO)?.let { accountId ->
                scheduler.scheduleNuvioProgress(
                    NuvioPlaybackProgressSchedule(
                        profileId = profile.id,
                        providerAccountId = accountId,
                        contentId = meta.id,
                        contentType = meta.type,
                        videoId = videoId,
                        position = positionMs,
                        duration = durationMs
                    )
                )
            }
        }
        if (allowPauseScrobble && profile.isProviderConnected(ThirdPartyProviderId.TRAKT)) {
            scheduleTraktScrobble(profile, meta, videoId, positionMs, durationMs, "pause")
        }
        if (allowPauseScrobble && profile.isProviderConnected(ThirdPartyProviderId.SIMKL)) {
            scheduleSimklScrobble(profile, meta, videoId, positionMs, durationMs, "pause")
        }
    }

    fun scheduleTraktScrobble(
        profile: UserProfile,
        meta: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: String
    ): Boolean {
        if (!profile.isProviderConnected(ThirdPartyProviderId.TRAKT) || durationMs <= 0L) return false
        val progress = (positionMs.toFloat() / durationMs.toFloat() * 100f).coerceIn(0f, 100f)
        return scheduleTraktScrobble(
            profile,
            meta.type,
            TraktIntegration.scrobbleMediaId(meta.id, videoId, meta.type),
            progress,
            action
        )
    }

    fun scheduleTraktScrobble(
        profile: UserProfile,
        mediaType: String,
        mediaId: String,
        progress: Float,
        action: String
    ): Boolean {
        if (!profile.isProviderConnected(ThirdPartyProviderId.TRAKT)) return false
        val accountId = profile.providerAccountId(ThirdPartyProviderId.TRAKT) ?: return false
        return scheduler.scheduleTraktScrobble(
            TraktScrobbleSchedule(profile.id, accountId, mediaType, mediaId, progress.coerceIn(0f, 100f), action)
        )
    }

    fun scheduleSimklScrobble(
        profile: UserProfile,
        meta: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: String
    ): Boolean {
        if (!profile.isProviderConnected(ThirdPartyProviderId.SIMKL) || durationMs <= 0L) return false
        return scheduler.scheduleSimklScrobble(
            SimklScrobbleSchedule(
                profileId = profile.id,
                providerAccountId = profile.providerAccountId(ThirdPartyProviderId.SIMKL) ?: return false,
                mediaType = meta.type,
                mediaId = TraktIntegration.scrobbleMediaId(meta.id, videoId, meta.type),
                action = action,
                positionMs = positionMs,
                durationMs = durationMs
            )
        )
    }

    /** Clear local display selection is handled by callers; remote progress is cleared everywhere. */
    suspend fun clearProgress(profile: UserProfile, meta: Meta): Boolean =
        thirdPartyProviderRepository.removeContinueWatchingFromConnected(profile, meta)

    /** Read-side Continue Watching still follows the user's selected source only. */
    fun cachedContinueWatching(profile: UserProfile): List<Meta> {
        val providerId = continueWatchingProvider(profile) ?: return emptyList()
        return thirdPartyProviderRepository.cached(profile, providerId)
            ?.continueWatching
            .orEmpty()
    }

    suspend fun pushWatched(
        profile: UserProfile,
        meta: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = thirdPartyProviderRepository.pushWatchedToConnected(
        profile = profile,
        item = meta,
        episodes = episodes,
        watched = watched
    )
}
