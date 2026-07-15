package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

internal class HomeFocusCoordinator(
    private val scope: CoroutineScope,
    private val focusedMovie: () -> Meta?,
    private val setFocusedMovie: (Meta?) -> Unit,
    private val setFocusedTrailer: (String?) -> Unit,
    private val setPreview: (String?) -> Unit,
    private val activeProfile: () -> UserProfile?,
    private val getConfiguredMetaDetail: suspend (String, String, String) -> HomeMetaDetailResult
) {
    private var focusJob: Job? = null

    fun onMovieFocused(movie: Meta) {
        if (focusedMovie()?.id == movie.id) return
        focusJob?.cancel()
        setPreview(null)

        focusJob = scope.launch {
            setFocusedMovie(movie)
            setFocusedTrailer(null)
            delay(500)

            try {
                if (!isActive) return@launch
                val detailResult = getConfiguredMetaDetail(movie.type, movie.id, activeProfile()?.safeLanguage ?: "en")
                val detail = detailResult.detail
                if (detail != null) {
                    setFocusedMovie(
                        movie.copy(
                            description = detail.description ?: movie.description,
                            background = detail.background,
                            imdbRating = detail.imdbRating ?: movie.imdbRating,
                            ratings = detail.ratings ?: movie.ratings,
                            releaseInfo = detail.releaseInfo ?: movie.releaseInfo,
                            runtime = detail.runtime ?: movie.runtime,
                            seasonsCount = detail.seasonsCount ?: detail.videos?.mapNotNull { it.season }?.filter { it > 0 }?.distinct()?.size,
                            episodesCount = detail.videos?.size,
                            cast = detail.cast
                        )
                    )
                }

                delay(1500)
                if (isActive) {
                    val previewUrl = detailResult.trailers.firstOrNull()?.url
                    setFocusedTrailer(previewUrl)
                    setPreview(previewUrl)
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}

internal data class HomeMetaDetailResult(
    val detail: MetaDetail?,
    val trailers: List<DetailTrailer>
)
