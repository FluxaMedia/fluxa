package com.fluxa.app.ui.catalog

import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.unit.Dp

enum class WindowWidthClass { Compact, Medium, Expanded }

data class CatalogGridLayoutSpec(
    val pageHorizontalPadding: Dp,
    val headerVerticalPadding: Dp,
    val titleSize: androidx.compose.ui.unit.TextUnit,
    val minimumCardWidth: Dp,
    val horizontalSpacing: Dp,
    val verticalSpacing: Dp
)

data class TopNavigationLayoutSpec(
    val horizontalPadding: Dp,
    val verticalPadding: Dp,
    val itemSpacing: Dp,
    val iconSize: Dp,
    val itemHorizontalPadding: Dp,
    val itemVerticalPadding: Dp,
    val labelSize: androidx.compose.ui.unit.TextUnit
)

val LocalWindowWidthClass = compositionLocalOf { WindowWidthClass.Compact }

fun widthClassFor(maxWidth: Dp): WindowWidthClass = when {
    maxWidth < FluxaUiLayoutTokens.Compact.Dp.breakpointMaxWidth -> WindowWidthClass.Compact
    maxWidth < FluxaUiLayoutTokens.Medium.Dp.breakpointMaxWidth -> WindowWidthClass.Medium
    else -> WindowWidthClass.Expanded
}

fun WindowWidthClass.gridColumns(): Int = when (this) {
    WindowWidthClass.Compact -> FluxaUiLayoutTokens.Compact.Int.gridColumns
    WindowWidthClass.Medium -> FluxaUiLayoutTokens.Medium.Int.gridColumns
    WindowWidthClass.Expanded -> FluxaUiLayoutTokens.Expanded.Int.gridColumns
}

@Composable
fun rememberCatalogGridLayoutSpec(): CatalogGridLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> CatalogGridLayoutSpec(
        FluxaUiLayoutTokens.Compact.Dp.pageHorizontalPadding,
        FluxaUiLayoutTokens.Compact.Dp.headerVerticalPadding,
        FluxaUiLayoutTokens.Compact.Sp.catalogTitleSize,
        FluxaUiLayoutTokens.Compact.Dp.minimumCardWidth,
        FluxaUiLayoutTokens.Compact.Dp.horizontalSpacing,
        FluxaUiLayoutTokens.Compact.Dp.verticalSpacing
    )
    WindowWidthClass.Medium -> CatalogGridLayoutSpec(
        FluxaUiLayoutTokens.Medium.Dp.pageHorizontalPadding,
        FluxaUiLayoutTokens.Medium.Dp.headerVerticalPadding,
        FluxaUiLayoutTokens.Medium.Sp.catalogTitleSize,
        FluxaUiLayoutTokens.Medium.Dp.minimumCardWidth,
        FluxaUiLayoutTokens.Medium.Dp.horizontalSpacing,
        FluxaUiLayoutTokens.Medium.Dp.verticalSpacing
    )
    WindowWidthClass.Expanded -> CatalogGridLayoutSpec(
        FluxaUiLayoutTokens.Expanded.Dp.pageHorizontalPadding,
        FluxaUiLayoutTokens.Expanded.Dp.headerVerticalPadding,
        FluxaUiLayoutTokens.Expanded.Sp.catalogTitleSize,
        FluxaUiLayoutTokens.Expanded.Dp.minimumCardWidth,
        FluxaUiLayoutTokens.Expanded.Dp.horizontalSpacing,
        FluxaUiLayoutTokens.Expanded.Dp.verticalSpacing
    )
}

@Composable
fun rememberTopNavigationLayoutSpec(): TopNavigationLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> TopNavigationLayoutSpec(
        FluxaUiLayoutTokens.Compact.Dp.navigationHorizontalPadding,
        FluxaUiLayoutTokens.Compact.Dp.navigationVerticalPadding,
        FluxaUiLayoutTokens.Compact.Dp.navigationItemSpacing,
        FluxaUiLayoutTokens.Compact.Dp.navigationIconSize,
        FluxaUiLayoutTokens.Compact.Dp.navigationItemHorizontalPadding,
        FluxaUiLayoutTokens.Compact.Dp.navigationItemVerticalPadding,
        FluxaUiLayoutTokens.Compact.Sp.navigationLabelSize
    )
    WindowWidthClass.Medium -> TopNavigationLayoutSpec(
        FluxaUiLayoutTokens.Medium.Dp.navigationHorizontalPadding,
        FluxaUiLayoutTokens.Medium.Dp.navigationVerticalPadding,
        FluxaUiLayoutTokens.Medium.Dp.navigationItemSpacing,
        FluxaUiLayoutTokens.Medium.Dp.navigationIconSize,
        FluxaUiLayoutTokens.Medium.Dp.navigationItemHorizontalPadding,
        FluxaUiLayoutTokens.Medium.Dp.navigationItemVerticalPadding,
        FluxaUiLayoutTokens.Medium.Sp.navigationLabelSize
    )
    WindowWidthClass.Expanded -> TopNavigationLayoutSpec(
        FluxaUiLayoutTokens.Expanded.Dp.navigationHorizontalPadding,
        FluxaUiLayoutTokens.Expanded.Dp.navigationVerticalPadding,
        FluxaUiLayoutTokens.Expanded.Dp.navigationItemSpacing,
        FluxaUiLayoutTokens.Expanded.Dp.navigationIconSize,
        FluxaUiLayoutTokens.Expanded.Dp.navigationItemHorizontalPadding,
        FluxaUiLayoutTokens.Expanded.Dp.navigationItemVerticalPadding,
        FluxaUiLayoutTokens.Expanded.Sp.navigationLabelSize
    )
}

@Composable
fun rememberCatalogGridCells(): GridCells {
    return GridCells.Fixed(LocalWindowWidthClass.current.gridColumns())
}
