package com.fluxa.app.shared.image

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale

@Composable
expect fun FluxaRemoteImage(
    imageUrl: String?,
    cacheKey: String?,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    onError: (() -> Unit)? = null
)
