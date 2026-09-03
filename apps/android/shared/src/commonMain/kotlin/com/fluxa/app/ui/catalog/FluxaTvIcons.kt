package com.fluxa.app.ui.catalog

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

object FluxaTvIcons {
    val ReleaseTimeline: ImageVector = ImageVector.Builder(
        name = "ReleaseTimeline",
        defaultWidth = 24.dp,
        defaultHeight = 24.dp,
        viewportWidth = 24f,
        viewportHeight = 24f
    ).apply {
        path(
            fill = null,
            stroke = SolidColor(Color.Black),
            strokeLineWidth = 1.8f,
            strokeLineCap = StrokeCap.Round,
            strokeLineJoin = StrokeJoin.Round
        ) {
            moveTo(6f, 5.5f)
            lineTo(18f, 5.5f)
            quadTo(20f, 5.5f, 20f, 7.5f)
            lineTo(20f, 18f)
            quadTo(20f, 20f, 18f, 20f)
            lineTo(6f, 20f)
            quadTo(4f, 20f, 4f, 18f)
            lineTo(4f, 7.5f)
            quadTo(4f, 5.5f, 6f, 5.5f)
            close()
            moveTo(4f, 9.5f)
            lineTo(20f, 9.5f)
            moveTo(8f, 3.5f)
            lineTo(8f, 7.5f)
            moveTo(16f, 3.5f)
            lineTo(16f, 7.5f)
            moveTo(8f, 14.5f)
            lineTo(13.5f, 14.5f)
            moveTo(16.25f, 14.5f)
            lineTo(16.25f, 14.5f)
        }
    }.build()
}
