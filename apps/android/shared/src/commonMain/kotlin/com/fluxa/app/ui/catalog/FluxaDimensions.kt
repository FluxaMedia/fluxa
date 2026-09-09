package com.fluxa.app.ui.catalog

import androidx.compose.ui.graphics.Color

/** Compatibility facade backed by shared/contracts/ui-tokens.json. */
object FluxaDimensions {
    object EpisodeCard {
        val mobileWidth = FluxaUiLayoutTokens.Mobile.Dp.episodeCardWidth
        val mobileHeight = FluxaUiLayoutTokens.Mobile.Dp.episodeCardHeight
        val desktopWidth = FluxaUiLayoutTokens.Desktop.Dp.episodeCardWidth
        val desktopHeight = FluxaUiLayoutTokens.Desktop.Dp.episodeCardHeight
        val tvWidth = FluxaUiLayoutTokens.Tv.Dp.episodeCardWidth
        val tvHeight = FluxaUiLayoutTokens.Tv.Dp.episodeCardHeight
    }

    object TvPosterCard {
        val width = FluxaUiLayoutTokens.Tv.Dp.posterCardWidth
        val height = FluxaUiLayoutTokens.Tv.Dp.posterCardHeight
    }

    val mobileBillboardHeight = FluxaUiLayoutTokens.Common.Dp.mobileBillboardHeight

    object PosterPresets {
        val xsmall = FluxaUiLayoutTokens.Common.Dp.posterXsmall
        val small = FluxaUiLayoutTokens.Common.Dp.posterSmall
        val medium = FluxaUiLayoutTokens.Common.Dp.posterMedium
        val large = FluxaUiLayoutTokens.Common.Dp.posterLarge
        val xlarge = FluxaUiLayoutTokens.Common.Dp.posterXlarge
        val heightRatio = FluxaUiLayoutTokens.Common.Number.posterHeightRatio
    }

    object CornerPresets {
        val sharp = FluxaUiLayoutTokens.Common.Dp.cornerSharp
        val classic = FluxaUiLayoutTokens.Common.Dp.cornerClassic
        val highlight = FluxaUiLayoutTokens.Common.Dp.cornerHighlight
        val soft = FluxaUiLayoutTokens.Common.Dp.cornerSoft
        val rounded = FluxaUiLayoutTokens.Common.Dp.cornerRounded
        val pill = FluxaUiLayoutTokens.Common.Dp.cornerPill
    }

    object HorizontalCard {
        val mobileBase = FluxaUiLayoutTokens.Mobile.Dp.horizontalCardBase
        val desktopBase = FluxaUiLayoutTokens.Desktop.Dp.horizontalCardBase
        val tvBase = FluxaUiLayoutTokens.Tv.Dp.horizontalCardBase
        val heightRatio = FluxaUiLayoutTokens.Common.Number.horizontalCardHeightRatio
        val deltaXsmall = FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaXsmall
        val deltaSmall = FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaSmall
        val deltaLarge = FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaLarge
        val deltaXlarge = FluxaUiLayoutTokens.Common.Dp.horizontalCardDeltaXlarge
    }

    val cardMetaBarHeight = FluxaUiLayoutTokens.Common.Dp.cardMetaBarHeight
    val cardMetaBarWithEpisodeLabelHeight = FluxaUiLayoutTokens.Common.Dp.cardMetaBarWithEpisodeLabelHeight
    val cardProgressBarHeight = FluxaUiLayoutTokens.Common.Dp.cardProgressBarHeight

    object LibraryListItem {
        val height = FluxaUiLayoutTokens.Common.Dp.libraryListItemHeight
        val thumbnailWidth = FluxaUiLayoutTokens.Common.Dp.libraryThumbnailWidth
        val rowCornerRadius = FluxaUiLayoutTokens.Common.Dp.libraryRowCornerRadius
        val thumbnailCornerRadius = FluxaUiLayoutTokens.Common.Dp.libraryThumbnailCornerRadius
    }

    object Profile {
        val avatarSize = FluxaUiLayoutTokens.Common.Dp.profileAvatarSize
    }

    object CardText {
        val titleSize = FluxaUiLayoutTokens.Common.Sp.cardTitleSize
        val subtitleSize = FluxaUiLayoutTokens.Common.Sp.cardSubtitleSize
        val coverEmojiSize = FluxaUiLayoutTokens.Common.Sp.cardCoverEmojiSize
        val coverFallbackSize = FluxaUiLayoutTokens.Common.Sp.cardCoverFallbackSize
    }

    object Alpha {
        const val emptyCardBackground = FluxaUiLayoutTokens.Common.Alpha.emptyCardBackground
        const val cardSubtitle = FluxaUiLayoutTokens.Common.Alpha.cardSubtitle
        const val progressBarTrack = FluxaUiLayoutTokens.Common.Alpha.progressBarTrack
        const val hairline = FluxaUiLayoutTokens.Common.Alpha.hairline
        const val subtleBorder = FluxaUiLayoutTokens.Common.Alpha.subtleBorder
        const val mediumBorder = FluxaUiLayoutTokens.Common.Alpha.mediumBorder
        const val dimText = FluxaUiLayoutTokens.Common.Alpha.dimText
        const val mutedText = FluxaUiLayoutTokens.Common.Alpha.mutedText
        const val upNextBadge = FluxaUiLayoutTokens.Common.Alpha.upNextBadge
        const val coverEmoji = FluxaUiLayoutTokens.Common.Alpha.coverEmoji
        const val coverFallbackText = FluxaUiLayoutTokens.Common.Alpha.coverFallbackText
        const val secondaryText = FluxaUiLayoutTokens.Common.Alpha.secondaryText
        const val faintText = FluxaUiLayoutTokens.Common.Alpha.faintText
        const val trackInactive = FluxaUiLayoutTokens.Common.Alpha.trackInactive
        const val mutedLabel = FluxaUiLayoutTokens.Common.Alpha.mutedLabel
        const val valueText = FluxaUiLayoutTokens.Common.Alpha.valueText
        const val placeholderText = FluxaUiLayoutTokens.Common.Alpha.placeholderText
        const val borderFaint = FluxaUiLayoutTokens.Common.Alpha.borderFaint
        const val iconMuted = FluxaUiLayoutTokens.Common.Alpha.iconMuted
    }

    object AnimDuration {
        val blink = FluxaUiLayoutTokens.Common.DurationMs.blink
        val quick = FluxaUiLayoutTokens.Common.DurationMs.quick
        val scaleAlpha = FluxaUiLayoutTokens.Common.DurationMs.scaleAlpha
        val fadeIn = FluxaUiLayoutTokens.Common.DurationMs.fadeIn
        val contentExpand = FluxaUiLayoutTokens.Common.DurationMs.contentExpand
        val cardFocusScale = FluxaUiLayoutTokens.Common.DurationMs.cardFocusScale
        val settingsExpand = FluxaUiLayoutTokens.Common.DurationMs.settingsExpand
        val settingsExpandAlt = FluxaUiLayoutTokens.Common.DurationMs.settingsExpandAlt
        val heightAnim = FluxaUiLayoutTokens.Common.DurationMs.heightAnim
        val heroSnap = FluxaUiLayoutTokens.Common.DurationMs.heroSnap
        val fadeOut = FluxaUiLayoutTokens.Common.DurationMs.fadeOut
        val routeExit = FluxaUiLayoutTokens.Common.DurationMs.routeExit
        val parentsContainer = FluxaUiLayoutTokens.Common.DurationMs.parentsContainer
        val parentsExpand = FluxaUiLayoutTokens.Common.DurationMs.parentsExpand
        val nextEpisode = FluxaUiLayoutTokens.Common.DurationMs.nextEpisode
        val progressRing = FluxaUiLayoutTokens.Common.DurationMs.progressRing
        val sidebarSlide = FluxaUiLayoutTokens.Common.DurationMs.sidebarSlide
        val heroReveal = FluxaUiLayoutTokens.Common.DurationMs.heroReveal
        val ambientColor = FluxaUiLayoutTokens.Common.DurationMs.ambientColor
        val loginPulse = FluxaUiLayoutTokens.Common.DurationMs.loginPulse
        val marquee = FluxaUiLayoutTokens.Common.DurationMs.marquee
    }

    object PlayerChrome {
        val topScrimHeight = FluxaUiLayoutTokens.Common.Dp.playerTopScrimHeight
        val bottomScrimHeight = FluxaUiLayoutTokens.Common.Dp.playerBottomScrimHeight
        const val topScrimAlpha = FluxaUiLayoutTokens.Common.Alpha.playerTopScrim
        const val bottomScrimAlpha = FluxaUiLayoutTokens.Common.Alpha.playerBottomScrim
        val seekTrackHeight = FluxaUiLayoutTokens.Common.Dp.playerSeekTrackHeight
        val seekTrackHeightDragging = FluxaUiLayoutTokens.Common.Dp.playerSeekTrackHeightDragging
        val edgeMargin = FluxaUiLayoutTokens.Common.Dp.playerEdgeMargin
        val iconSize = FluxaUiLayoutTokens.Common.Dp.playerIconSize
        val pillCornerRadius = FluxaUiLayoutTokens.Common.Dp.playerPillCornerRadius
        const val chromeDimAlpha = FluxaUiLayoutTokens.Common.Alpha.playerChromeDim
        const val textAlphaPrimary = 1.0f
        const val textAlphaSecondary = FluxaUiLayoutTokens.Common.Alpha.playerTextSecondary
        const val textAlphaDisabled = FluxaUiLayoutTokens.Common.Alpha.playerTextDisabled
        val deckCornerRadius = FluxaUiLayoutTokens.Common.Dp.playerDeckCornerRadius
        val deckPadding = FluxaUiLayoutTokens.Common.Dp.playerDeckPadding
        val deckActionRowSpacing = FluxaUiLayoutTokens.Common.Dp.playerDeckActionRowSpacing
        val deckBackground = Color(0xE6101418)
        val deckDivider = Color.White.copy(alpha = FluxaUiLayoutTokens.Common.Alpha.playerDeckDivider)
        val titleTextSize = FluxaUiLayoutTokens.Common.Sp.playerTitleTextSize
        val metaTextSize = FluxaUiLayoutTokens.Common.Sp.playerMetaTextSize
        val timeTextSize = FluxaUiLayoutTokens.Common.Sp.playerTimeTextSize
        val actionLabelTextSize = FluxaUiLayoutTokens.Common.Sp.playerActionLabelTextSize
        val sidebarTitleTextSize = FluxaUiLayoutTokens.Common.Sp.playerSidebarTitleTextSize
        val sidebarRowTextSize = FluxaUiLayoutTokens.Common.Sp.playerSidebarRowTextSize
        val sidebarRowSubtitleTextSize = FluxaUiLayoutTokens.Common.Sp.playerSidebarRowSubtitleTextSize
    }

    val mobileFocusBorderStroke = FluxaUiLayoutTokens.Common.Dp.mobileFocusBorderStroke
    val tvFocusBorderStroke = FluxaUiLayoutTokens.Common.Dp.tvFocusBorderStroke
    val cardFocusedScale = FluxaUiLayoutTokens.Common.Number.cardFocusedScale
}
