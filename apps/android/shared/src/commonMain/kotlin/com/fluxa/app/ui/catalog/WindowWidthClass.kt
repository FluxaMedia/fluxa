package com.fluxa.app.ui.catalog

import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

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
    maxWidth < 600.dp -> WindowWidthClass.Compact
    maxWidth < 840.dp -> WindowWidthClass.Medium
    else -> WindowWidthClass.Expanded
}

fun WindowWidthClass.gridColumns(): Int = when (this) {
    WindowWidthClass.Compact -> 3
    WindowWidthClass.Medium -> 5
    WindowWidthClass.Expanded -> 7
}

@Composable
fun rememberCatalogGridLayoutSpec(): CatalogGridLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> CatalogGridLayoutSpec(16.dp, 12.dp, 24.sp, 150.dp, 12.dp, 16.dp)
    WindowWidthClass.Medium -> CatalogGridLayoutSpec(28.dp, 20.dp, 26.sp, 160.dp, 16.dp, 20.dp)
    WindowWidthClass.Expanded -> CatalogGridLayoutSpec(48.dp, 28.dp, 28.sp, 170.dp, 20.dp, 24.dp)
}

@Composable
fun rememberTopNavigationLayoutSpec(): TopNavigationLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> TopNavigationLayoutSpec(20.dp, 10.dp, 6.dp, 24.dp, 12.dp, 8.dp, 13.sp)
    WindowWidthClass.Medium -> TopNavigationLayoutSpec(32.dp, 12.dp, 8.dp, 26.dp, 14.dp, 9.dp, 14.sp)
    WindowWidthClass.Expanded -> TopNavigationLayoutSpec(48.dp, 16.dp, 12.dp, 28.dp, 16.dp, 10.dp, 15.sp)
}

@Composable
fun rememberCatalogGridCells(): GridCells {
    return GridCells.Fixed(LocalWindowWidthClass.current.gridColumns())
}
