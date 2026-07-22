package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta

data class EditorialPickSpec(
    val title: String,
    val minYear: Int
)

object HomeBillboardRanking {
    fun scoreCandidate(meta: Meta): Int =
        FluxaCoreNative.homeBillboardCandidateScore(meta, ReleaseDateUtils.daysSince(meta.released))

    fun hasBackdropCandidate(meta: Meta): Boolean = FluxaCoreNative.homeBillboardHasBackdrop(meta)

    fun visualScore(meta: Meta): Int = FluxaCoreNative.homeBillboardVisualScore(meta)

    fun editorialMatchScore(meta: Meta, spec: EditorialPickSpec): Int =
        FluxaCoreNative.homeBillboardEditorialMatchScore(meta, spec.minYear)

    fun normalizeTitle(value: String): String = FluxaCoreNative.homeBillboardNormalizedTitle(value)

    fun contentIdentityKey(meta: Meta): String = FluxaCoreNative.homeBillboardIdentityKey(meta)
}
