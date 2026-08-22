package com.fluxa.app.shared.feature.player

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun PlayerSeekFeedback(direction: Int, seekMs: Long) {
    val seconds = (seekMs / 1000L).coerceAtLeast(1L)
    val transition = rememberInfiniteTransition(label = "seek")
    val chevronShift by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(220, easing = FastOutSlowInEasing), RepeatMode.Restart),
        label = "chevronShift"
    )
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Canvas(Modifier.size(120.dp, 82.dp)) {
            val sign = if (direction > 0) 1f else -1f
            val stroke = size.minDimension * 0.075f
            val centerY = size.height / 2f
            val baseX = size.width / 2f - sign * size.width * 0.14f
            repeat(3) { index ->
                val phase = (chevronShift + index * 0.28f) % 1f
                val alpha = 0.22f + phase * 0.58f
                val x = baseX + sign * (index * size.width * 0.12f + phase * size.width * 0.05f)
                drawLine(Color.White.copy(alpha), Offset(x - sign * size.width * 0.06f, centerY - size.height * 0.12f), Offset(x + sign * size.width * 0.06f, centerY), stroke, StrokeCap.Round)
                drawLine(Color.White.copy(alpha), Offset(x + sign * size.width * 0.06f, centerY), Offset(x - sign * size.width * 0.06f, centerY + size.height * 0.12f), stroke, StrokeCap.Round)
            }
        }
        Text(
            text = if (direction > 0) "+$seconds" else "-$seconds",
            color = Color.White,
            fontWeight = FontWeight.Black,
            fontSize = 22.sp,
            style = androidx.compose.ui.text.TextStyle(shadow = Shadow(Color.Black.copy(alpha = 0.55f), Offset(2f, 4f), 8f))
        )
    }
}

fun formatPlayerTime(ms: Long): String {
    val totalSeconds = ms / 1000L
    val hours = totalSeconds / 3600L
    val minutes = ((totalSeconds % 3600L) / 60L).toString().padStart(2, '0')
    val seconds = (totalSeconds % 60L).toString().padStart(2, '0')
    return if (hours > 0L) "${hours.toString().padStart(2, '0')}:$minutes:$seconds" else "$minutes:$seconds"
}
