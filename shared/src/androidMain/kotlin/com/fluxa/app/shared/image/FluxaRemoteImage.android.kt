package com.fluxa.app.shared.image

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.request.ImageRequest
import coil3.request.crossfade

@Composable
actual fun FluxaRemoteImage(
    imageUrl: String?,
    cacheKey: String?,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    onError: (() -> Unit)?
) {
    val context = LocalPlatformContext.current
    val request = remember(context, imageUrl, cacheKey) {
        ImageRequest.Builder(context)
            .data(imageUrl)
            .crossfade(false)
            .memoryCacheKey(cacheKey)
            .diskCacheKey(cacheKey)
            .build()
    }
    AsyncImage(
        model = request,
        contentDescription = contentDescription,
        modifier = modifier,
        contentScale = contentScale,
        onError = { onError?.invoke() }
    )
}
