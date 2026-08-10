package com.fluxa.app.shared.image

import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale

@Composable
expect fun FluxaRemoteImage(
    imageUrl: String?,
    cacheKey: String?,
    diskCacheKey: String? = null,
    requestWidthPx: Int? = null,
    requestHeightPx: Int? = null,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    alignment: Alignment = Alignment.Center,
    onError: (() -> Unit)? = null,
    trimTransparentPadding: Boolean = false
)

fun sanitizeImageUrl(url: String): String = url.replace(" ", "%20")

private val animatedImageExtensions = setOf("gif", "webp")

fun isAnimatedImageUrl(url: String): Boolean =
    url.substringBefore('?').substringBefore('#').substringAfterLast('.', "").lowercase() in animatedImageExtensions
