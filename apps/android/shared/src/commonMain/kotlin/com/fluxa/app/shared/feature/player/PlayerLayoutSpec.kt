package com.fluxa.app.shared.feature.player

import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass

internal data class PlayerLayoutSpec(
    val edgePadding: Dp,
    val titleSize: TextUnit,
    val centerHorizontalPadding: Dp,
    val centerVerticalPadding: Dp,
    val centerSpacing: Dp,
    val playButtonSize: Dp,
    val playIconSize: Dp,
    val panelHorizontalPadding: Dp,
    val panelVerticalPadding: Dp,
    val skipMinWidth: Dp,
    val skipHorizontalPadding: Dp,
    val skipVerticalPadding: Dp,
    val skipFontSize: TextUnit,
    val nextThumbnailSize: Dp,
    val nextCardWidth: Dp,
    val nextCardRadius: Dp,
    val nextSpacing: Dp,
    val nextTitleSize: TextUnit,
    val nextSubtitleSize: TextUnit,
    val nextIconSize: Dp,
    val loadingContainerWidth: Dp,
    val skipBottomInset: Dp,
    val toastTopInset: Dp,
    val seekFeedbackWidth: Dp
)

internal data class PlayerSidebarLayoutSpec(
    val cardWidth: Dp,
    val listMaxHeight: Dp,
    val backButtonSize: Dp,
    val backIconSize: Dp,
    val titleSize: TextUnit,
    val rowMinHeight: Dp,
    val rowTextSize: TextUnit
)

@Composable
internal fun rememberPlayerLayoutSpec(): PlayerLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> PlayerLayoutSpec(16.dp, 18.sp, 12.dp, 10.dp, 14.dp, 60.dp, 28.dp, 14.dp, 12.dp, 108.dp, 16.dp, 8.dp, 13.sp, 46.dp, 240.dp, 12.dp, 10.dp, 10.sp, 10.sp, 20.dp, 280.dp, 76.dp, 34.dp, 180.dp)
    WindowWidthClass.Medium -> PlayerLayoutSpec(22.dp, 21.sp, 15.dp, 12.dp, 17.dp, 68.dp, 33.dp, 16.dp, 13.dp, 140.dp, 22.dp, 11.dp, 14.sp, 58.dp, 300.dp, 14.dp, 12.dp, 12.sp, 11.sp, 24.dp, 380.dp, 92.dp, 44.dp, 240.dp)
    WindowWidthClass.Expanded -> PlayerLayoutSpec(28.dp, 24.sp, 18.dp, 14.dp, 20.dp, 78.dp, 38.dp, 18.dp, 14.dp, 160.dp, 28.dp, 13.dp, 16.sp, 74.dp, 364.dp, 14.dp, 12.dp, 14.sp, 13.sp, 28.dp, 500.dp, 92.dp, 54.dp, 300.dp)
}

@Composable
internal fun rememberPlayerSidebarLayoutSpec(): PlayerSidebarLayoutSpec = when (LocalWindowWidthClass.current) {
    WindowWidthClass.Compact -> PlayerSidebarLayoutSpec(340.dp, 440.dp, 30.dp, 16.dp, 14.sp, 46.dp, 14.sp)
    WindowWidthClass.Medium -> PlayerSidebarLayoutSpec(360.dp, 420.dp, 32.dp, 18.dp, 15.sp, 48.dp, 14.sp)
    WindowWidthClass.Expanded -> PlayerSidebarLayoutSpec(420.dp, 400.dp, 34.dp, 20.dp, 16.sp, 52.dp, 15.sp)
}
