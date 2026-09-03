package com.fluxa.app.shared.feature.catalog

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import com.fluxa.app.shared.feature.player.TrailerCue

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    onPlayClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier,
    trailerItemId: String?,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    focusedItemOverride: CatalogItemUiModel?,
    posterFocused: Boolean,
    onActiveItemChanged: (CatalogItemUiModel) -> Unit,
    onFocusChanged: (Boolean) -> Unit,
    focusRequester: FocusRequester?,
    downFocusRequester: FocusRequester?,
    onDownPressed: () -> Unit,
    leftFocusRequester: FocusRequester?
) {
}
