package com.fluxa.app.ui.catalog

import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

enum class WindowWidthClass { Compact, Medium, Expanded }

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
fun rememberCatalogGridCells(): GridCells {
    return if (LocalDeviceType.current == DeviceType.Desktop) {
        GridCells.Adaptive(minSize = FluxaDimensions.PosterPresets.medium)
    } else {
        GridCells.Fixed(LocalWindowWidthClass.current.gridColumns())
    }
}
