package com.fluxa.app.ui.catalog

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke

@Composable
internal fun DefaultProfileAvatar(
    modifier: Modifier = Modifier,
    tint: Color = Color.White
) {
    Canvas(modifier = modifier.fillMaxSize()) {
        val w = size.width
        val h = size.height
        val strokeWidth = size.minDimension * 0.075f
        drawCircle(
            color = tint,
            radius = w * 0.085f,
            center = Offset(w * 0.30f, h * 0.37f)
        )
        drawCircle(
            color = tint,
            radius = w * 0.075f,
            center = Offset(w * 0.67f, h * 0.34f)
        )
        val smile = Path().apply {
            moveTo(w * 0.22f, h * 0.62f)
            cubicTo(w * 0.20f, h * 0.54f, w * 0.26f, h * 0.57f, w * 0.30f, h * 0.65f)
            cubicTo(w * 0.42f, h * 0.75f, w * 0.66f, h * 0.68f, w * 0.74f, h * 0.57f)
        }
        drawPath(
            path = smile,
            color = tint,
            style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
        )
    }
}
