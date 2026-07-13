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
    val base = if (deviceType == DeviceType.TV) FluxaDimensions.HorizontalCard.tvBase else FluxaDimensions.HorizontalCard.mobileBase
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
