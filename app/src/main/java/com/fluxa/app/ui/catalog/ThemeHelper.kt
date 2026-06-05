package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.graphics.Bitmap
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.palette.graphics.Palette
import coil3.BitmapImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.allowHardware
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

data class ContentColors(
    val dominant: Color = Color(0xFFE85D3F),
    val darkMuted: Color = Color(0xFF12161D),
    val lightVibrant: Color = Color(0xFFF4F1EA)
)

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
                dominant = palette.vibrantSwatch?.let { Color(it.rgb) } ?: Color(0xFFE85D3F),
                darkMuted = palette.darkMutedSwatch?.let { Color(it.rgb) } ?: Color(0xFF12161D),
                lightVibrant = palette.lightVibrantSwatch?.let { Color(it.rgb) } ?: Color(0xFFF4F1EA)
            ).also { colorCache.put(url, it) }
        } catch (e: Exception) {
            ContentColors()
        }
    }
}
