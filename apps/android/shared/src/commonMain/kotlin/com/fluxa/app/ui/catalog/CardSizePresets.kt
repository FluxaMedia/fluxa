package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

fun posterCardWidth(value: String): Dp = with(FluxaDimensions.PosterPresets) {
    when (value) {
        "xsmall" -> xsmall
        "small" -> small
        "large" -> large
        "xlarge" -> xlarge
        else -> medium
    }
}

fun posterCardHeight(value: String): Dp = posterCardWidth(value) * FluxaDimensions.PosterPresets.heightRatio

fun horizontalCardWidth(value: String, deviceType: DeviceType): Dp {
    val base = when (deviceType) {
        DeviceType.Mobile -> FluxaUiLayoutTokens.Mobile.Dp.horizontalCardBase
        DeviceType.Desktop -> FluxaUiLayoutTokens.Desktop.Dp.horizontalCardBase
        DeviceType.TV -> FluxaUiLayoutTokens.Tv.Dp.horizontalCardBase
    }
    return base + horizontalCardSizeDelta(value)
}

fun horizontalCardWidth(value: String, widthClass: WindowWidthClass): Dp {
    val base = when (widthClass) {
        WindowWidthClass.Compact -> FluxaUiLayoutTokens.Compact.Dp.horizontalCardBase
        WindowWidthClass.Medium -> FluxaUiLayoutTokens.Medium.Dp.horizontalCardBase
        WindowWidthClass.Expanded -> FluxaUiLayoutTokens.Expanded.Dp.horizontalCardBase
    }
    return base + horizontalCardSizeDelta(value)
}

private fun horizontalCardSizeDelta(value: String): Dp = when (value) {
    "xsmall" -> FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaXsmall
    "small" -> FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaSmall
    "large" -> FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaLarge
    "xlarge" -> FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaXlarge
    else -> 0.dp
}

fun horizontalCardHeight(value: String, deviceType: DeviceType): Dp = horizontalCardWidth(value, deviceType) * FluxaDimensions.HorizontalCard.heightRatio

fun horizontalCardHeight(value: String, widthClass: WindowWidthClass): Dp = horizontalCardWidth(value, widthClass) * FluxaDimensions.HorizontalCard.heightRatio

fun cardCornerRadius(preset: String): Dp = when (preset) {
    "sharp" -> FluxaUiLayoutTokens.Common.Dp.cornerSharp
    "classic" -> FluxaUiLayoutTokens.Common.Dp.cornerClassic
    "soft" -> FluxaUiLayoutTokens.Common.Dp.cornerSoft
    "rounded" -> FluxaUiLayoutTokens.Common.Dp.cornerRounded
    "pill" -> FluxaUiLayoutTokens.Common.Dp.cornerPill
    else -> FluxaUiLayoutTokens.Common.Dp.cardCornerDefault
}

fun cardRowSpacing(preset: String): Dp = when (preset) {
    "small" -> FluxaUiLayoutTokens.Common.Dp.cardRowSpacingSmall
    "medium" -> FluxaUiLayoutTokens.Common.Dp.cardRowSpacingMedium
    "large" -> FluxaUiLayoutTokens.Common.Dp.cardRowSpacingLarge
    else -> FluxaUiLayoutTokens.Common.Dp.cardRowSpacingMedium
}
