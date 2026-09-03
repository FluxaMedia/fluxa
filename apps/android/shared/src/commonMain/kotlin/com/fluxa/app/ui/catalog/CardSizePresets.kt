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
    return horizontalCardWidth(
        value = value,
        widthClass = if (deviceType == DeviceType.TV) WindowWidthClass.Expanded else WindowWidthClass.Compact
    )
}

fun horizontalCardWidth(value: String, widthClass: WindowWidthClass): Dp {
    val base = when (widthClass) {
        WindowWidthClass.Compact -> FluxaDimensions.HorizontalCard.mobileBase
        WindowWidthClass.Medium -> 210.dp
        WindowWidthClass.Expanded -> FluxaDimensions.HorizontalCard.tvBase
    }
    val delta = with(FluxaDimensions.HorizontalCard) {
        when (value) {
            "xsmall" -> deltaXsmall
            "small" -> deltaSmall
            "large" -> deltaLarge
            "xlarge" -> deltaXlarge
            else -> 0.dp
        }
    }
    return base + delta
}

fun horizontalCardHeight(value: String, deviceType: DeviceType): Dp = horizontalCardWidth(value, deviceType) * FluxaDimensions.HorizontalCard.heightRatio

fun horizontalCardHeight(value: String, widthClass: WindowWidthClass): Dp = horizontalCardWidth(value, widthClass) * FluxaDimensions.HorizontalCard.heightRatio

fun cardCornerRadius(preset: String): Dp = when (preset) {
    "sharp" -> 0.dp
    "classic" -> 4.dp
    "soft" -> 8.dp
    "rounded" -> 14.dp
    "pill" -> 22.dp
    else -> 8.dp
}

fun cardRowSpacing(preset: String): Dp = when (preset) {
    "small" -> 6.dp
    "medium" -> 12.dp
    "large" -> 20.dp
    else -> 12.dp
}
