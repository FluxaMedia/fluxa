package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.player.AudioCodecBadge
import com.fluxa.app.player.VideoFormatBadge

private val shape = RoundedCornerShape(4.dp)

@Composable
private fun TextBadge(label: String, bg: Color, fg: Color, border: Color = Color.Transparent) {
    Box(
        modifier = Modifier
            .clip(shape)
            .background(bg)
            .then(if (border != Color.Transparent) Modifier.border(0.5.dp, border, shape) else Modifier)
            .padding(horizontal = 6.dp, vertical = 3.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = fg,
            fontSize = 9.sp,
            fontWeight = FontWeight.ExtraBold,
            letterSpacing = 0.5.sp,
            maxLines = 1
        )
    }
}

// Dolby
@Composable fun DolbyAtmosBadge()       = TextBadge("Atmos",  Color(0xFF0A0A0A), Color.White,        Color.White.copy(alpha = 0.20f))
@Composable fun DolbyDigitalPlusBadge() = TextBadge("DD+",    Color(0xFF0A0A0A), Color.White,        Color.White.copy(alpha = 0.20f))
@Composable fun DolbyDigitalBadge()     = TextBadge("DD",     Color(0xFF0A0A0A), Color.White,        Color.White.copy(alpha = 0.20f))
@Composable fun DolbyTrueHDBadge()      = TextBadge("TrueHD", Color(0xFF0A0A0A), Color.White,        Color.White.copy(alpha = 0.20f))

// DTS
@Composable fun DtsXBadge()  = TextBadge("DTS:X",  Color(0xFF00172B), Color(0xFF00AEEF), Color(0xFF00AEEF).copy(alpha = 0.40f))
@Composable fun DtsHdBadge() = TextBadge("DTS-HD", Color(0xFF00172B), Color(0xFF00AEEF), Color(0xFF00AEEF).copy(alpha = 0.40f))
@Composable fun DtsBadge()   = TextBadge("DTS",    Color(0xFF009CDE), Color.White)

// HDR / display
@Composable fun DolbyVisionBadge() = TextBadge("Dolby Vision", Color(0xFF00172B), Color(0xFF4DB3FF), Color(0xFF4DB3FF).copy(alpha = 0.40f))
@Composable fun Hdr10PlusBadge()   = TextBadge("HDR10+",       Color(0xFF1A1000), Color(0xFFE6A817), Color(0xFFE6A817).copy(alpha = 0.50f))
@Composable fun Hdr10Badge()        = TextBadge("HDR10",        Color(0xFF1A1000), Color(0xFFE6A817), Color(0xFFE6A817).copy(alpha = 0.50f))
@Composable fun HlgBadge()          = TextBadge("HLG",          Color(0xFF0D1A0D), Color(0xFF7BC67E), Color(0xFF7BC67E).copy(alpha = 0.50f))

@Composable
fun VideoFormatBadgeView(badge: VideoFormatBadge) {
    when (badge) {
        VideoFormatBadge.DolbyVision -> DolbyVisionBadge()
        VideoFormatBadge.Hdr10Plus   -> Hdr10PlusBadge()
        VideoFormatBadge.Hdr10       -> Hdr10Badge()
        VideoFormatBadge.Hlg         -> HlgBadge()
    }
}

@Composable
fun AudioCodecBadgeView(badge: AudioCodecBadge) {
    when (badge) {
        AudioCodecBadge.DolbyAtmos       -> DolbyAtmosBadge()
        AudioCodecBadge.DolbyDigitalPlus -> DolbyDigitalPlusBadge()
        AudioCodecBadge.DolbyDigital     -> DolbyDigitalBadge()
        AudioCodecBadge.DolbyTrueHD      -> DolbyTrueHDBadge()
        AudioCodecBadge.DtsX             -> DtsXBadge()
        AudioCodecBadge.DtsHd            -> DtsHdBadge()
        AudioCodecBadge.Dts              -> DtsBadge()
    }
}
