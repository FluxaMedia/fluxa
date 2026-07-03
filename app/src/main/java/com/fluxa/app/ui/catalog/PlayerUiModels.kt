package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable
import com.fluxa.app.data.remote.Meta

@Immutable
internal data class PlayerContentUiModel(
    val id: String,
    val type: String,
    val title: String,
    val logo: String,
    val background: String,
    val releaseInfo: String?,
    val runtime: String?
) {
    val isSeries: Boolean get() = type == "series"
}

internal fun Meta.toPlayerContentUiModel(): PlayerContentUiModel =
    PlayerContentUiModel(
        id = id,
        type = type,
        title = name,
        logo = logo.orEmpty(),
        background = background.orEmpty(),
        releaseInfo = releaseInfo,
        runtime = runtime
    )
