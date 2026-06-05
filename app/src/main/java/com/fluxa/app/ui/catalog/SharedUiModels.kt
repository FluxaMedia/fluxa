package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream

data class StreamUiModel(
    val header: String,
    val body: String?
)

fun Stream.toStreamUiModel() = StreamUiModel(
    header = streamSourceHeader(),
    body = streamRawBody()
)

data class SimilarItemUiModel(
    val id: String,
    val type: String,
    val title: String,
    val poster: String?
)

fun Meta.toSimilarItemUiModel() = SimilarItemUiModel(
    id = id,
    type = type,
    title = name,
    poster = poster
)

data class BillboardUiModel(
    val id: String,
    val type: String,
    val title: String,
    val description: String?,
    val releaseInfo: String?,
    val genres: List<String>,
    val runtime: String?,
    val seasonsCount: Int?,
    val ageRating: String?,
    val backgroundUrl: String?,
    val logoFallbackUrl: String?
)

fun Meta.toBillboardUiModel() = BillboardUiModel(
    id = id,
    type = type,
    title = name,
    description = description,
    releaseInfo = releaseInfo,
    genres = genres ?: emptyList(),
    runtime = runtime,
    seasonsCount = seasonsCount,
    ageRating = ageRating,
    backgroundUrl = background,
    logoFallbackUrl = logo
)
