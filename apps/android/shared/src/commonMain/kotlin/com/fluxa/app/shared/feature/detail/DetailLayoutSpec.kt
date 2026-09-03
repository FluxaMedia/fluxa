package com.fluxa.app.shared.feature.detail

import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass

internal data class DetailLayoutSpec(
    val heroHeight: Dp,
    val contentStartInset: Dp,
    val contentMaxWidth: Dp?,
    val usesTwoPane: Boolean
)

@Composable
internal fun rememberDetailLayoutSpec(): DetailLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> DetailLayoutSpec(560.dp, 20.dp, null, false)
    WindowWidthClass.Medium -> DetailLayoutSpec(480.dp, 32.dp, 560.dp, false)
    WindowWidthClass.Expanded -> DetailLayoutSpec(420.dp, 96.dp, 420.dp, true)
}
