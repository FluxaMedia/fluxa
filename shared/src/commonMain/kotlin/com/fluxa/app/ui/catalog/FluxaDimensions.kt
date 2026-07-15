package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

object FluxaDimensions {

    object EpisodeCard {
        val mobileWidth = 244.dp
        val mobileHeight = 148.dp
        val tvWidth = 356.dp
        val tvHeight = 208.dp
    }

    object TvPosterCard {
        val width = 136.dp
        val height = 204.dp
    }

    val mobileBillboardHeight = 540.dp

    object PosterPresets {
        val xsmall = 96.dp
        val small = 112.dp
        val medium = 128.dp
        val large = 148.dp
        val xlarge = 176.dp
        const val heightRatio = 1.5f
    }

    object CornerPresets {
        val sharp = 2.dp
        val classic = 8.dp
        val soft = 12.dp
        val rounded = 18.dp
        val pill = 28.dp
    }

    object HorizontalCard {
        val mobileBase = 166.dp
        val tvBase = 260.dp
        const val heightRatio = 0.56f
        val deltaXsmall = (-25).dp
        val deltaSmall = (-13).dp
        val deltaLarge = 31.dp
        val deltaXlarge = 61.dp
    }

    val cardMetaBarHeight = 28.dp
    val cardMetaBarWithEpisodeLabelHeight = 50.dp

    val cardProgressBarHeight = 4.dp

    object LibraryListItem {
        val height = 132.dp
        val thumbnailWidth = 112.dp
        val rowCornerRadius = 14.dp
        val thumbnailCornerRadius = 10.dp
    }

    object Profile {
        val avatarSize = 120.dp
    }

    object CardText {
        val titleSize = 12.sp
        val subtitleSize = 10.sp
        val coverEmojiSize = 42.sp
        val coverFallbackSize = 48.sp
    }

    object Alpha {
        const val emptyCardBackground = 0.05f
        const val cardSubtitle = 0.58f
        const val progressBarTrack = 0.38f
        const val subtleBorder = 0.08f
        const val mediumBorder = 0.16f
        const val dimText = 0.48f
        const val mutedText = 0.56f
        const val upNextBadge = 0.72f
        const val coverEmoji = 0.82f
        const val coverFallbackText = 0.2f
    }

    object AnimDuration {
        const val blink = 90
        const val quick = 140
        const val scaleAlpha = 180
        const val fadeIn = 200
        const val contentExpand = 220
        const val cardFocusScale = 240
        const val settingsExpand = 240
        const val settingsExpandAlt = 260
        const val heightAnim = 130
        const val heroSnap = 150
        const val fadeOut = 150
        const val routeExit = 160
        const val parentsContainer = 300
        const val parentsExpand = 400
        const val nextEpisode = 560
        const val progressRing = 520
        const val sidebarSlide = 620
        const val heroReveal = 650
        const val ambientColor = 700
        const val loginPulse = 1000
        const val marquee = 1120
    }

    object PlayerChrome {
        val topScrimHeight = 160.dp
        val bottomScrimHeight = 230.dp
        const val topScrimAlpha = 0.72f
        const val bottomScrimAlpha = 0.86f
        val seekTrackHeight = 4.dp
    }

    val mobileFocusBorderStroke = 2.dp
    val tvFocusBorderStroke = 3.dp

    const val cardFocusedScale = 1.12f
}
