package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.player.PlayerContentUiModel

internal fun Meta.toPlayerContentUiModel(): PlayerContentUiModel =
    PlayerContentUiModel(
        id = id,
        type = type,
        title = name,
        logoUrl = logo,
        backgroundUrl = background,
        releaseInfo = releaseInfo,
        runtime = runtime
    )
