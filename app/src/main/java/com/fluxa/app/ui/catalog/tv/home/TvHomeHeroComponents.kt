@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.tv.material3.Button
import androidx.tv.material3.ButtonDefaults
import androidx.tv.material3.Icon
import androidx.tv.material3.Text
import coil3.compose.AsyncImage
import com.fluxa.app.R
import com.fluxa.app.data.remote.Meta

@Composable
internal fun HomeHeroBackdrop(
    selectedContent: Meta?,
    activePreviewUrl: String?,
    sharedPlayer: androidx.media3.exoplayer.ExoPlayer?,
    seasonPostersOnHero: Boolean = true
) {
    val youtubeId = remember(activePreviewUrl) { activePreviewUrl?.extractYoutubeVideoId() }
    Box(modifier = Modifier.fillMaxSize()) {
        when {
            youtubeId != null -> {
                YoutubeHeroPreview(
                    youtubeId = youtubeId,
                    modifier = Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(0.72f)
                        .align(Alignment.TopEnd)
                        .alpha(0.94f)
                )
            }
            activePreviewUrl?.isDirectVideoPreviewUrl() == true && sharedPlayer != null -> {
                AndroidView(
                    factory = { context ->
                        androidx.media3.ui.PlayerView(context).apply {
                            useController = false
                            player = sharedPlayer
                        }
                    },
                    update = { playerView ->
                        playerView.player = sharedPlayer
                    },
                    modifier = Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(0.72f)
                        .align(Alignment.TopEnd)
                        .alpha(0.94f)
                )
            }
            activePreviewUrl?.isNotBlank() == true -> {
                WebHeroPreview(
                    url = activePreviewUrl,
                    modifier = Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(0.72f)
                        .align(Alignment.TopEnd)
                        .alpha(0.94f)
                )
            }
            else -> {
                AsyncImage(
                    model = selectedContent?.homeHeroBackdrop(seasonPostersOnHero),
                    contentDescription = null,
                    modifier = Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(0.72f)
                        .align(Alignment.TopEnd)
                        .alpha(0.94f),
                    contentScale = ContentScale.Crop
                )
            }
        }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.horizontalGradient(
                        colors = listOf(
                            Color(0xFF090B12),
                            Color(0xFF090B12).copy(alpha = 0.98f),
                            Color(0xFF090B12).copy(alpha = 0.82f),
                            Color(0xFF090B12).copy(alpha = 0.34f),
                            Color.Transparent
                        )
                    )
                )
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            Color(0xFF090B12).copy(alpha = 0.28f),
                            Color.Transparent,
                            Color(0xFF090B12).copy(alpha = 0.92f)
                        )
                    )
                )
        )
        Box(
            modifier = Modifier
                .fillMaxHeight()
                .width(240.dp)
                .background(
                    Brush.horizontalGradient(
                        colors = listOf(
                            Color(0xFF090B12),
                            Color(0xFF090B12).copy(alpha = 0.88f),
                            Color.Transparent
                        )
                    )
                )
        )
    }
}

@Composable
private fun YoutubeHeroPreview(
    youtubeId: String,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val html = remember(youtubeId) { youtubeEmbedHtml(youtubeId) }
    AndroidView(
        factory = {
            android.webkit.WebView(context).apply {
                settings.javaScriptEnabled = true
                settings.mediaPlaybackRequiresUserGesture = false
                settings.domStorageEnabled = true
                settings.loadsImagesAutomatically = true
                webChromeClient = android.webkit.WebChromeClient()
                tag = youtubeId
                loadDataWithBaseURL("https://www.youtube.com", html, "text/html", "utf-8", null)
            }
        },
        update = { webView ->
            if (webView.tag != youtubeId) {
                webView.tag = youtubeId
                webView.loadDataWithBaseURL("https://www.youtube.com", html, "text/html", "utf-8", null)
            }
        },
        modifier = modifier,
        onRelease = { webView ->
            webView.stopLoading()
            webView.loadUrl("about:blank")
            webView.webChromeClient = null
            webView.destroy()
        }
    )
}

@Composable
private fun WebHeroPreview(
    url: String,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    AndroidView(
        factory = {
            android.webkit.WebView(context).apply {
                settings.javaScriptEnabled = true
                settings.mediaPlaybackRequiresUserGesture = false
                settings.domStorageEnabled = true
                settings.loadsImagesAutomatically = true
                webChromeClient = android.webkit.WebChromeClient()
                tag = url
                loadUrl(url)
            }
        },
        update = { webView ->
            if (webView.tag != url) {
                webView.tag = url
                webView.loadUrl(url)
            }
        },
        modifier = modifier,
        onRelease = { webView ->
            webView.stopLoading()
            webView.loadUrl("about:blank")
            webView.webChromeClient = null
            webView.destroy()
        }
    )
}

private fun youtubeEmbedHtml(youtubeId: String): String {
    val cleanId = youtubeId.filter { it.isLetterOrDigit() || it == '_' || it == '-' }
    return """
        <!doctype html>
        <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <style>
                    html, body, iframe {
                        width: 100%;
                        height: 100%;
                        margin: 0;
                        padding: 0;
                        overflow: hidden;
                        background: #000;
                    }
                </style>
            </head>
            <body>
                <iframe
                    src="https://www.youtube.com/embed/$cleanId?autoplay=1&mute=1&controls=0&playsinline=1&rel=0&modestbranding=1&loop=1&playlist=$cleanId"
                    frameborder="0"
                    allow="autoplay; encrypted-media; picture-in-picture"
                    allowfullscreen>
                </iframe>
            </body>
        </html>
    """.trimIndent()
}

@Composable
internal fun HomeHeroPanel(
    movie: Meta,
    logoUrl: String?,
    lang: String,
    playButtonFocusRequester: FocusRequester,
    belowHeroFocusRequester: FocusRequester?,
    modifier: Modifier = Modifier,
    onPlayClick: () -> Unit,
    onInfoClick: () -> Unit
) {
    val logoCandidates = remember(movie.id, movie.logo, logoUrl) {
        listOfNotNull(logoUrl, movie.logo).distinct()
    }

    androidx.compose.foundation.layout.Column(modifier = modifier.widthIn(max = 560.dp)) {
        if (logoCandidates.isNotEmpty()) {
            AsyncImage(
                model = logoCandidates.first(),
                contentDescription = movie.name,
                modifier = Modifier
                    .height(94.dp)
                    .widthIn(max = 360.dp),
                contentScale = ContentScale.Fit
            )
        } else {
            Text(
                text = movie.name,
                color = Color(0xFFFFD94B),
                fontSize = 54.sp,
                lineHeight = 56.sp,
                fontWeight = FontWeight.Black,
                style = TextStyle(
                    shadow = Shadow(
                        color = Color.Black.copy(alpha = 0.55f),
                        blurRadius = 20f
                    )
                )
            )
        }

        Spacer(modifier = Modifier.height(18.dp))
        HeroMetaLine(movie = movie, lang = lang)
        Spacer(modifier = Modifier.height(10.dp))
        HeroRatingLine(movie = movie)
        Spacer(modifier = Modifier.height(18.dp))

        if (!movie.description.isNullOrBlank()) {
            Text(
                text = movie.description.orEmpty(),
                color = Color.White.copy(alpha = 0.88f),
                fontSize = 18.sp,
                lineHeight = 27.sp,
                maxLines = 4,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 500.dp)
            )
            Spacer(modifier = Modifier.height(22.dp))
        }

        Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            Button(
                onClick = onPlayClick,
                modifier = Modifier
                    .focusRequester(playButtonFocusRequester)
                    .then(
                        if (belowHeroFocusRequester != null) {
                            Modifier.focusProperties { down = belowHeroFocusRequester }
                        } else {
                            Modifier
                        }
                    ),
                colors = ButtonDefaults.colors(
                    containerColor = Color.White,
                    contentColor = Color.Black
                ),
                scale = ButtonDefaults.scale(focusedScale = 1.05f)
            ) {
                Icon(
                    imageVector = FluxaIcons.PlayArrow,
                    contentDescription = null,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = AppStrings.t(lang, "common.play"), fontWeight = FontWeight.Bold)
            }
            Button(
                onClick = onInfoClick,
                modifier = Modifier.then(
                    if (belowHeroFocusRequester != null) {
                        Modifier.focusProperties { down = belowHeroFocusRequester }
                    } else {
                        Modifier
                    }
                ),
                colors = ButtonDefaults.colors(
                    containerColor = Color.White.copy(alpha = 0.12f),
                    contentColor = Color.White
                ),
                scale = ButtonDefaults.scale(focusedScale = 1.05f)
            ) {
                Text(text = AppStrings.t(lang, "common.details"), fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

@Composable
private fun HeroMetaLine(movie: Meta, lang: String) {
    val pieces = buildList {
        add(
            when (movie.type.lowercase()) {
                "series" -> AppStrings.t(lang, "auto.series")
                "movie" -> AppStrings.t(lang, "auto.movie")
                else -> movie.type.replaceFirstChar { it.uppercase() }
            }
        )
        movie.genres?.firstOrNull()?.takeIf { it.isNotBlank() }?.let { add(it) }
        movie.releaseInfo?.takeIf { it.isNotBlank() }?.let { add(it.take(4)) }
        movie.runtime?.takeIf { it.isNotBlank() }?.let { add(it) }
    }

    Text(
        text = pieces.joinToString("    "),
        color = Color.White.copy(alpha = 0.82f),
        fontSize = 16.sp,
        fontWeight = FontWeight.Medium
    )
}

@Composable
private fun HeroRatingLine(movie: Meta) {
    val rating = movie.imdbRating?.takeIf { it.isNotBlank() } ?: return
    ImdbRatingBadge(rating = rating)
}

@Composable
private fun ImdbRatingBadge(rating: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier
            .clip(RoundedCornerShape(11.dp))
            .background(Color.White.copy(alpha = 0.08f))
            .padding(start = 10.dp, end = 12.dp, top = 8.dp, bottom = 8.dp)
    ) {
        Image(
            painter = painterResource(id = R.drawable.imdb_logo),
            contentDescription = null,
            modifier = Modifier.height(18.dp)
        )
        Text(
            text = rating,
            color = Color.White,
            fontSize = 15.sp,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
internal fun HeroBadge(text: String, background: Color, content: Color) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(8.dp))
            .background(background)
            .padding(horizontal = 10.dp, vertical = 4.dp)
    ) {
        Text(
            text = text,
            color = content,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold
        )
    }
}
