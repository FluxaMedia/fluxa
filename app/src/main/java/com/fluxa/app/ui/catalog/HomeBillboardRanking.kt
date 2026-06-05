package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.ContentIdentity

internal data class EditorialPickSpec(
    val title: String,
    val minYear: Int
)

internal object HomeBillboardRanking {
    fun scoreCandidate(meta: Meta): Int {
        return FluxaCoreNative.billboardScoreCandidate(meta, ReleaseDateUtils.daysSince(meta.released))
    }

    fun hasBackdropCandidate(meta: Meta): Boolean {
        return FluxaCoreNative.billboardHasBackdropCandidate(meta)
    }

    fun visualScore(meta: Meta): Int {
        return FluxaCoreNative.billboardVisualScore(meta)
    }

    fun editorialMatchScore(meta: Meta, spec: EditorialPickSpec): Int {
        return FluxaCoreNative.billboardEditorialMatchScore(meta, spec)
    }

    fun normalizeTitle(value: String): String {
        return ContentIdentity.normalizedBillboardTitle(value)
    }

    fun contentIdentityKey(meta: Meta): String {
        return ContentIdentity.billboardKey(meta)
    }
}
