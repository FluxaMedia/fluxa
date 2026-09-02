package com.fluxa.app.shared.feature.catalog

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import com.fluxa.app.shared.feature.player.TrailerCue

@Composable
expect fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier = Modifier,
    trailerItemId: String? = null,
    trailerUrl: String? = null,
    trailerSubtitleCues: List<TrailerCue> = emptyList(),
    onFocusChanged: (Boolean) -> Unit = {},
    focusRequester: FocusRequester? = null,
    downFocusRequester: FocusRequester? = null,
    onDownPressed: () -> Unit = {}
)
