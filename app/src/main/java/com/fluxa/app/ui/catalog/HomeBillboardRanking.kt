package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.contentBillboardKey
import com.fluxa.app.domain.normalizedBillboardTitle

data class EditorialPickSpec(
    val title: String,
    val minYear: Int
)

object HomeBillboardRanking {
    fun scoreCandidate(meta: Meta): Int {
        val daysSinceRelease = ReleaseDateUtils.daysSince(meta.released)
        val releaseBoost = when {
            daysSinceRelease == null -> 0
            daysSinceRelease < 0 -> 40
            daysSinceRelease <= 14 -> 440
            daysSinceRelease <= 45 -> 280
            daysSinceRelease <= 120 -> 120
            else -> 0
        }
        val typeBoost = if (meta.type == "series") 320 else 140
        val rankBoost = meta.rank?.let { (220 - (it - 1) * 18).coerceAtLeast(0) } ?: 0
        val ratingBoost = ((meta.imdbRating?.toFloatOrNull() ?: 0f) * 22).toInt()
        val recommendationBoost = if (meta.reason.isNullOrEmpty()) 0 else 180
        val editorialBoost = if (meta.reason == "EDITORIAL_SPOTLIGHT") 520 else 0
        val backdropBoost = when {
            hasBackdropCandidate(meta) -> 260
            !meta.poster.isNullOrEmpty() -> 40
            else -> -240
        }
        return typeBoost + releaseBoost + rankBoost + ratingBoost + recommendationBoost + editorialBoost + backdropBoost
    }

    fun hasBackdropCandidate(meta: Meta): Boolean {
        val background = meta.background.orEmpty()
        return background.isNotEmpty() && !background.equals(meta.poster.orEmpty(), ignoreCase = true)
    }

    fun visualScore(meta: Meta): Int {
        var score = if (hasBackdropCandidate(meta)) 320 else -160
        if (!meta.logo.isNullOrEmpty()) score += 120
        if (!meta.description.isNullOrEmpty()) score += 30
        return score
    }

    fun editorialMatchScore(meta: Meta, spec: EditorialPickSpec): Int {
        val releaseYear = meta.releaseInfo?.toIntOrNull() ?: 0
        val yearBoost = if (releaseYear >= spec.minYear) 400 else 0
        val ratingBoost = ((meta.imdbRating?.toFloatOrNull() ?: 0f) * 20).toInt()
        val rankBoost = meta.rank?.let { (180 - it * 12).coerceAtLeast(0) } ?: 0
        return yearBoost + ratingBoost + rankBoost
    }

    fun normalizeTitle(value: String): String = normalizedBillboardTitle(value)

    fun contentIdentityKey(meta: Meta): String = contentBillboardKey(meta)
}
