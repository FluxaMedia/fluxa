package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import android.util.Log
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import com.fluxa.app.BuildConfig
import androidx.hilt.work.HiltWorker
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject

@HiltWorker
class TraktScrobbleWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    profileManager: ProfileManager,
    private val api: TraktApi
) : ProviderSyncPushWorker(appContext, params, profileManager) {

    override val providerName = "trakt"

    override suspend fun doWork(): Result {
        if (!TraktIntegration.hasClient(BuildConfig.TRAKT_CLIENT_ID)) return Result.success()

        val profileId = inputData.getString(KEY_PROFILE_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val mediaType = inputData.getString(KEY_MEDIA_TYPE)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val mediaId = inputData.getString(KEY_MEDIA_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val action = inputData.getString(KEY_ACTION)?.takeIf { it in setOf("start", "pause", "stop") } ?: return Result.failure()
        val progress = inputData.getFloat(KEY_PROGRESS, -1f).takeIf { it >= 0f }?.coerceIn(0f, 100f) ?: return Result.failure()

        val profile = requireProfile(profileId) ?: return Result.failure()
        val token = resolveAccessToken(profileManager, profile, api) ?: return Result.failure()
        val request = buildRequest(mediaType, mediaId, progress) ?: return Result.failure()

        Log.d(
            "TraktScrobbleWorker",
            "Scrobble request action=$action media_type=$mediaType media_id=$mediaId progress=${"%.4f".format(java.util.Locale.US, progress)} url=${TraktIntegration.scrobbleUrl(action)} headers=${TraktIntegration.traktHeadersForLog(token, BuildConfig.TRAKT_CLIENT_ID)} body=${TraktIntegration.toLogJson(request)}"
        )

        return runCatching {
            when (action) {
                "start" -> api.scrobbleStart(TraktIntegration.bearer(token), BuildConfig.TRAKT_CLIENT_ID, request)
                "pause" -> api.scrobblePause(TraktIntegration.bearer(token), BuildConfig.TRAKT_CLIENT_ID, request)
                "stop" -> api.scrobbleStop(TraktIntegration.bearer(token), BuildConfig.TRAKT_CLIENT_ID, request)
                else -> null
            }
        }.fold(
            onSuccess = { response ->
                when {
                    response == null -> {
                        onSyncFailure(profileId)
                        Result.failure()
                    }
                    response.isSuccessful -> {
                        Log.d(
                            "TraktScrobbleWorker",
                            "Scrobble response action=$action media_id=$mediaId success=true http=${response.code()}"
                        )
                        onSyncSuccess(profileId)
                        Result.success()
                    }
                    response.code() == 409 -> {
                        val body = runCatching { response.errorBody()?.string()?.take(1000) }.getOrNull()
                        Log.w("TraktScrobbleWorker", "Scrobble response action=$action media_id=$mediaId success=true duplicate=true http=409 error_body=${body.orEmpty()}")
                        onSyncSuccess(profileId)
                        Result.success()
                    }
                    response.code() == 429 || response.code() >= 500 -> {
                        onSyncFailure(profileId)
                        Result.retry()
                    }
                    response.code() == 401 && runAttemptCount == 0 -> Result.retry()
                    else -> {
                        val body = runCatching { response.errorBody()?.string()?.take(1000) }.getOrNull()
                        Log.w("TraktScrobbleWorker", "Scrobble response action=$action media_id=$mediaId success=false http=${response.code()} error_body=${body.orEmpty()}")
                        onSyncFailure(profileId)
                        Result.failure()
                    }
                }
            },
            onFailure = { error ->
                Log.w("TraktScrobbleWorker", "Scrobble $action failed for $mediaId", error)
                onSyncFailure(profileId)
                Result.retry()
            }
        )
    }

    private suspend fun resolveAccessToken(
        profileManager: ProfileManager,
        profile: UserProfile,
        api: TraktApi
    ): String? {
        val token = profile.traktAccessToken?.takeIf { it.isNotBlank() }
        val refreshToken = profile.traktRefreshToken?.takeIf { it.isNotBlank() }
        val shouldRefresh = refreshToken != null &&
            (token == null || profile.safeTraktTokenExpiresAt <= System.currentTimeMillis() + 60_000L)
        if (!shouldRefresh) return token

        return runCatching {
            api.refreshToken(
                TraktRefreshTokenRequest(
                    refresh_token = refreshToken,
                    client_id = BuildConfig.TRAKT_CLIENT_ID,
                    client_secret = BuildConfig.TRAKT_CLIENT_SECRET,
                    redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
                )
            )
        }.map { response ->
            val updated = profile.copy(
                traktAccessToken = response.accessToken,
                traktRefreshToken = response.refreshToken,
                traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
            )
            profileManager.saveProfile(updated)
            response.accessToken
        }.getOrElse { error ->
            val status = (error as? retrofit2.HttpException)?.code()
            if (status == 400 || status == 401) {
                profileManager.saveProfile(
                    profile.copy(
                        traktAccessToken = null,
                        traktRefreshToken = null,
                        traktTokenExpiresAt = null
                    )
                )
            }
            null
        }
    }

    private fun buildRequest(mediaType: String, mediaId: String, progress: Float): TraktScrobbleRequest? {
        val ids = TraktIntegration.idsFromContentId(mediaId) ?: return null
        return if (mediaType == "movie") {
            TraktScrobbleRequest(movie = TraktSummary(null, null, ids), progress = progress)
        } else {
            val episode = TraktIntegration.episodeLocator(mediaId) ?: return null
            TraktScrobbleRequest(
                show = TraktSummary(null, null, ids),
                episode = TraktScrobbleEpisode(season = episode.season, number = episode.episode),
                progress = progress
            )
        }
    }

    companion object {
        private const val KEY_PROFILE_ID = "profile_id"
        private const val KEY_MEDIA_TYPE = "media_type"
        private const val KEY_MEDIA_ID = "media_id"
        private const val KEY_PROGRESS = "progress"
        private const val KEY_ACTION = "action"

        fun enqueue(
            context: Context,
            profileId: String,
            mediaType: String,
            mediaId: String,
            progress: Float,
            action: String
        ) {
            if (profileId.isBlank() || mediaId.isBlank() || action !in setOf("start", "pause", "stop")) return
            val inputData = workDataOf(
                KEY_PROFILE_ID to profileId,
                KEY_MEDIA_TYPE to mediaType,
                KEY_MEDIA_ID to mediaId,
                KEY_PROGRESS to progress.coerceIn(0f, 100f),
                KEY_ACTION to action
            )
            ProviderSyncPushWorker.enqueueUnique<TraktScrobbleWorker>(
                context,
                "trakt_scrobble_${profileId}_${mediaType}_${mediaId}",
                inputData,
                expedited = true
            )
        }
    }
}
