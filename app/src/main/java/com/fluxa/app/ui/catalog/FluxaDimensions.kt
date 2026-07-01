package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * Single source of truth for all tweakable UI dimensions, durations, and style values.
 * Change a value here to affect every screen that uses it.
 */
object FluxaDimensions {

    // ── Episode / continue-watching card ─────────────────────────────────────
    object EpisodeCard {
        val mobileWidth = 244.dp
        val mobileHeight = 148.dp
        val tvWidth = 356.dp
        val tvHeight = 208.dp
    }

    // ── TV poster card (non-episode vertical poster) ──────────────────────────
    object TvPosterCard {
        val width = 136.dp
        val height = 204.dp
    }

    // ── Home billboard / hero ─────────────────────────────────────────────────
    val mobileBillboardHeight = 540.dp

    // ── Poster card size presets ──────────────────────────────────────────────
    // posterCardWidth(preset) and posterCardHeight(preset) in HomeScreen.kt consume these.
    object PosterPresets {
        val xsmall = 96.dp
        val small = 112.dp
        val medium = 128.dp
        val large = 148.dp
        val xlarge = 176.dp
        const val heightRatio = 1.5f
    }

    // ── Poster corner-radius presets ──────────────────────────────────────────
    // posterCornerRadius(preset) in HomeScreen.kt consumes these.
    object CornerPresets {
        val sharp = 2.dp
        val classic = 8.dp
        val soft = 12.dp     // default / "medium"
        val rounded = 18.dp
        val pill = 28.dp
    }

    // ── Horizontal card (backdrop thumbnail) ─────────────────────────────────
    // horizontalCardWidth/Height functions in HomeScreen.kt consume these.
    object HorizontalCard {
        val mobileBase = 166.dp
        val tvBase = 260.dp
        const val heightRatio = 0.56f
        val deltaXsmall = (-25).dp
        val deltaSmall = (-13).dp
        val deltaLarge = 31.dp
        val deltaXlarge = 61.dp
    }

    // ── Card meta bar (title + year row rendered below the image) ─────────────
    val cardMetaBarHeight = 42.dp
    val cardMetaBarWithEpisodeLabelHeight = 50.dp

    // ── In-card progress strip ────────────────────────────────────────────────
    val cardProgressBarHeight = 4.dp

    // ── Library list-item ─────────────────────────────────────────────────────
    object LibraryListItem {
        val height = 132.dp
        val thumbnailWidth = 112.dp
        val rowCornerRadius = 14.dp
        val thumbnailCornerRadius = 10.dp
    }

    // ── Profile screens ───────────────────────────────────────────────────────
    object Profile {
        val avatarSize = 120.dp
    }

    // ── Card text sizes ───────────────────────────────────────────────────────
    object CardText {
        val titleSize = 12.sp
        val subtitleSize = 10.sp
        val coverEmojiSize = 42.sp
        val coverFallbackSize = 48.sp
    }

    // ── Common alpha values ───────────────────────────────────────────────────
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

    // ── Animation durations (milliseconds) ───────────────────────────────────
    object AnimDuration {
        const val cardFocusScale = 240
        const val contentExpand = 220
        const val heroSnap = 150
        const val scaleAlpha = 180
        const val heightAnim = 130
        const val settingsExpand = 240
        const val settingsExpandAlt = 260
        const val heroReveal = 650
        const val fadeIn = 200
        const val fadeOut = 150
        const val ambientColor = 700
    }

    // ── Focus indicator stroke ────────────────────────────────────────────────
    val mobileFocusBorderStroke = 2.dp
    val tvFocusBorderStroke = 3.dp

    // ── Card focus scale factor ───────────────────────────────────────────────
    const val cardFocusedScale = 1.12f
}
