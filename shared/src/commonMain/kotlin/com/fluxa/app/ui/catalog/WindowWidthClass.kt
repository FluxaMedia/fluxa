package com.fluxa.app.ui.catalog

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
