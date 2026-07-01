package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.graphics.Bitmap
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.platform.LocalContext
import androidx.palette.graphics.Palette
import coil3.BitmapImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.allowHardware
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

data class ContentColors(
    val dominant: Color = FluxaColors.accent,
    val darkMuted: Color = FluxaColors.surface,
    val lightVibrant: Color = FluxaColors.textPrimary
)

@Composable
fun rememberAmbientHeroColor(artworkUrl: String?): Color {
    val context = LocalContext.current
    val colors by produceState(ContentColors(), artworkUrl) {
        value = ThemeHelper.extractColors(context, artworkUrl)
    }
    val target = lerp(colors.darkMuted, FluxaColors.background, 0.35f)
    return animateColorAsState(target, tween(FluxaDimensions.AnimDuration.ambientColor), label = "ambientHero").value
}

object ThemeHelper {
    private val colorCache = android.util.LruCache<String, ContentColors>(80)

    suspend fun extractColors(context: android.content.Context, url: String?): ContentColors = withContext(Dispatchers.IO) {
        if (url == null) return@withContext ContentColors()

        colorCache.get(url)?.let { return@withContext it }
        
        try {
            val request = ImageRequest.Builder(context)
                .data(url)
                .allowHardware(false) // Required for Palette
                .build()
            
            val result = context.imageLoader.execute(request)
            val bitmap = (result.image as? BitmapImage)?.bitmap ?: return@withContext ContentColors()
            
            val palette = Palette.from(bitmap).generate()
            
            ContentColors(
                dominant = palette.vibrantSwatch?.let { Color(it.rgb) } ?: FluxaColors.accent,
                darkMuted = palette.darkMutedSwatch?.let { Color(it.rgb) } ?: FluxaColors.surface,
                lightVibrant = palette.lightVibrantSwatch?.let { Color(it.rgb) } ?: FluxaColors.textPrimary
            ).also { colorCache.put(url, it) }
        } catch (e: Exception) {
            ContentColors()
        }
    }
}
