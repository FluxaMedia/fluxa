import Foundation

// Generated from shared/contracts/ui-tokens.json. Do not edit.
enum FluxaUiLayoutTokens {
  enum Common {
    enum Dp {
        static let mobileBillboardHeight = CGFloat(540)
        static let contentHeaderTopMobile = CGFloat(86)
        static let contentHeaderTop = CGFloat(110)
        static let detailHeaderTopMobile = CGFloat(82)
        static let detailHeaderTop = CGFloat(116)
        static let detailPosterWidthMobile = CGFloat(132)
        static let detailPosterWidthDesktop = CGFloat(220)
        static let detailPosterWidthTv = CGFloat(270)
        static let detailContentGap = CGFloat(28)
        static let detailMobileContentGap = CGFloat(22)
        static let detailActionGap = CGFloat(8)
        static let detailActionTopMobileOffset = CGFloat(182)
        static let detailActionTopOffset = CGFloat(188)
        static let detailSimilarTopMobileOffset = CGFloat(300)
        static let detailSimilarTopOffset = CGFloat(34)
        static let detailPlayWidth = CGFloat(118)
        static let detailWatchlistWidth = CGFloat(148)
        static let detailBackWidth = CGFloat(92)
        static let homeHeroContentMaxWidthDesktop = CGFloat(980)
        static let homeHeroLogoMaxWidthDesktop = CGFloat(820)
        static let homeHeroLogoHeightDesktop = CGFloat(148)
        static let homeHeroSynopsisHeightDesktop = CGFloat(88)
        static let librarySearchReservedWidth = CGFloat(150)
        static let librarySearchMinWidth = CGFloat(100)
        static let librarySearchMaxWidth = CGFloat(360)
        static let librarySortWidth = CGFloat(130)
        static let librarySortMinWidth = CGFloat(50)
        static let calendarPanelWidthDesktop = CGFloat(360)
        static let calendarPanelWidthTv = CGFloat(440)
        static let screenCardMinWidth = CGFloat(120)
        static let discoverCardTitleOffset = CGFloat(46)
        static let discoverCardSubtitleOffset = CGFloat(25)
        static let settingsContentOffset = CGFloat(220)
        static let settingsNavWidth = CGFloat(196)
        static let settingsScreenPaddingDesktop = CGFloat(48)
        static let settingsNavWidthDesktop = CGFloat(268)
        static let settingsPanelGapDesktop = CGFloat(32)
        static let settingsContentMaxWidthDesktop = CGFloat(2400)
        static let settingsNavItemHeightDesktop = CGFloat(44)
        static let settingsNavItemGapDesktop = CGFloat(6)
        static let settingsCardPaddingDesktop = CGFloat(24)
        static let settingsTitleTopDesktop = CGFloat(18)
        static let settingsDescriptionTopDesktop = CGFloat(54)
        static let settingsRowTopDesktop = CGFloat(76)
        static let settingsRowInsetDesktop = CGFloat(24)
        static let settingsRowHeightDesktop = CGFloat(44)
        static let settingsRowSpacingDesktop = CGFloat(56)
        static let settingsToggleWidthDesktop = CGFloat(48)
        static let settingsToggleHeightDesktop = CGFloat(26)
        static let settingsToggleKnobRadiusDesktop = CGFloat(9)
        static let settingsToggleOnKnobOffsetDesktop = CGFloat(13)
        static let settingsToggleOffKnobOffsetDesktop = CGFloat(43)
        static let settingsExtendedTop = CGFloat(104)
        static let settingsExtendedLineSpacing = CGFloat(34)
        static let settingsExtendedLineSpacingTv = CGFloat(42)
        static let settingsExtendedLineInset = CGFloat(20)
        static let settingsExtendedInputHeight = CGFloat(40)
        static let settingsExtendedActionHeight = CGFloat(38)
        static let settingsExtendedActionGap = CGFloat(10)
        static let settingsAddonActionWidth = CGFloat(148)
        static let settingsPluginActionWidth = CGFloat(156)
        static let settingsSmallActionWidth = CGFloat(76)
        static let settingsSmallActionHeight = CGFloat(34)
        static let settingsScraperActionWidth = CGFloat(96)
        static let settingsRepoRightInset = CGFloat(80)
        static let settingsScraperRightInset = CGFloat(104)
        static let screenControlHeight = CGFloat(36)
        static let screenControlRadius = CGFloat(9)
        static let focusRingExpand = CGFloat(3)
        static let focusRingRadius = CGFloat(10)
        static let focusRingWidth = CGFloat(2.5)
        static let cardOverlayHeight = CGFloat(82)
        static let cardContentPadding = CGFloat(10)
        static let settingsCardGap = CGFloat(18)
        static let settingsCardHeightMobile = CGFloat(168)
        static let settingsCardHeightDesktop = CGFloat(154)
        static let settingsCardPadding = CGFloat(16)
        static let settingsTitleTop = CGFloat(14)
        static let settingsDescriptionTop = CGFloat(39)
        static let settingsRowInset = CGFloat(14)
        static let settingsRowTop = CGFloat(67)
        static let settingsToggleRightInset = CGFloat(44)
        static let settingsToggleTopInset = CGFloat(10)
        static let settingsToggleOnKnobOffset = CGFloat(14)
        static let settingsToggleOffKnobOffset = CGFloat(34)
        static let settingsRowHeight = CGFloat(32)
        static let settingsRowSpacing = CGFloat(38)
        static let settingsToggleWidth = CGFloat(40)
        static let settingsToggleHeight = CGFloat(20)
        static let settingsToggleKnobRadius = CGFloat(7)
        static let calendarGridGapMobile = CGFloat(5)
        static let calendarGridGap = CGFloat(8)
        static let calendarCellHeightMobile = CGFloat(76)
        static let calendarCellHeightDesktop = CGFloat(96)
        static let calendarCellHeightTv = CGFloat(112)
        static let calendarCellRadius = CGFloat(9)
        static let calendarDayThumbWidth = CGFloat(23)
        static let calendarDayThumbHeight = CGFloat(32)
        static let libraryEmptyOffset = CGFloat(55)
        static let discoverEmptyOffset = CGFloat(70)
        static let calendarEmptyOffset = CGFloat(150)
        static let calendarDayPadding = CGFloat(8)
        static let calendarDayTop = CGFloat(7)
        static let calendarEntryTop = CGFloat(31)
        static let calendarEntryLineHeight = CGFloat(19)
        static let calendarThumbRightInset = CGFloat(30)
        static let calendarThumbTop = CGFloat(7)
        static let detailErrorGap = CGFloat(12)
        static let detailStreamGap = CGFloat(18)
        static let similarOverlayHeight = CGFloat(30)
        static let similarTextOffset = CGFloat(20)
        static let cardMetaBarHeight = CGFloat(28)
        static let cardMetaBarWithEpisodeLabelHeight = CGFloat(50)
        static let cardProgressBarHeight = CGFloat(4)
        static let libraryListItemHeight = CGFloat(132)
        static let libraryThumbnailWidth = CGFloat(112)
        static let libraryRowCornerRadius = CGFloat(14)
        static let libraryThumbnailCornerRadius = CGFloat(10)
        static let profileAvatarSize = CGFloat(120)
        static let posterXsmall = CGFloat(96)
        static let posterSmall = CGFloat(112)
        static let posterMedium = CGFloat(128)
        static let posterLarge = CGFloat(148)
        static let posterXlarge = CGFloat(176)
        static let cornerSharp = CGFloat(2)
        static let cornerClassic = CGFloat(8)
        static let cornerHighlight = CGFloat(10)
        static let cornerSoft = CGFloat(12)
        static let cornerRounded = CGFloat(18)
        static let cornerPill = CGFloat(28)
        static let horizontalCardDeltaXsmall = CGFloat(-25)
        static let horizontalCardDeltaSmall = CGFloat(-13)
        static let horizontalCardDeltaLarge = CGFloat(31)
        static let horizontalCardDeltaXlarge = CGFloat(61)
        static let playerTopScrimHeight = CGFloat(160)
        static let playerBottomScrimHeight = CGFloat(230)
        static let playerSeekTrackHeight = CGFloat(5)
        static let playerSeekTrackHeightDragging = CGFloat(7)
        static let playerEdgeMargin = CGFloat(20)
        static let playerIconSize = CGFloat(22)
        static let playerPillCornerRadius = CGFloat(999)
        static let playerDeckCornerRadius = CGFloat(22)
        static let playerDeckPadding = CGFloat(14)
        static let playerDeckActionRowSpacing = CGFloat(22)
        static let mobileFocusBorderStroke = CGFloat(2)
        static let tvFocusBorderStroke = CGFloat(3)
        static let cardCornerDefault = CGFloat(8)
        static let cardRowSpacingSmall = CGFloat(6)
        static let cardRowSpacingMedium = CGFloat(12)
        static let cardRowSpacingLarge = CGFloat(20)
    }
    enum Sp {
        static let screenTitleSizeMobile = CGFloat(32)
        static let screenTitleSize = CGFloat(34)
        static let screenTitleSizeTv = CGFloat(42)
        static let screenBodySize = CGFloat(15)
        static let screenBodySizeTv = CGFloat(18)
        static let screenSectionTitleSize = CGFloat(20)
        static let settingsSectionTitleSizeDesktop = CGFloat(28)
        static let screenSectionTitleSizeTv = CGFloat(24)
        static let screenCardTitleSize = CGFloat(15)
        static let screenCardTitleSizeTv = CGFloat(17)
        static let screenCardSubtitleSize = CGFloat(12)
        static let homeHeroSynopsisSizeDesktop = CGFloat(20)
        static let settingsDescriptionSizeDesktop = CGFloat(14)
        static let settingsRowLabelSizeDesktop = CGFloat(18)
        static let settingsRowValueSizeDesktop = CGFloat(15)
        static let cardTitleSize = CGFloat(12)
        static let cardSubtitleSize = CGFloat(10)
        static let cardCoverEmojiSize = CGFloat(42)
        static let cardCoverFallbackSize = CGFloat(48)
        static let playerTitleTextSize = CGFloat(15)
        static let playerMetaTextSize = CGFloat(12)
        static let playerTimeTextSize = CGFloat(13)
        static let playerActionLabelTextSize = CGFloat(11)
        static let playerSidebarTitleTextSize = CGFloat(15)
        static let playerSidebarRowTextSize = CGFloat(14)
        static let playerSidebarRowSubtitleTextSize = CGFloat(11)
        static let playerDeckTitleTextSize = CGFloat(15)
        static let playerDeckMetaTextSize = CGFloat(12)
        static let playerDeckTimeTextSize = CGFloat(13)
        static let playerDeckActionLabelTextSize = CGFloat(11)
    }
    enum DurationMs {
        static let blink = 90
        static let quick = 140
        static let scaleAlpha = 180
        static let fadeIn = 200
        static let contentExpand = 220
        static let cardFocusScale = 240
        static let settingsExpand = 240
        static let settingsExpandAlt = 260
        static let heightAnim = 130
        static let heroSnap = 150
        static let fadeOut = 150
        static let routeExit = 160
        static let parentsContainer = 300
        static let parentsExpand = 400
        static let nextEpisode = 560
        static let progressRing = 520
        static let sidebarSlide = 620
        static let heroReveal = 650
        static let ambientColor = 700
        static let loginPulse = 1000
        static let marquee = 1120
    }
    enum Number {
        static let posterHeightRatio = CGFloat(1.5)
        static let horizontalCardHeightRatio = CGFloat(0.56)
        static let cardFocusedScale = CGFloat(1.12)
    }
    enum Alpha {
        static let emptyCardBackground = CGFloat(0.05)
        static let cardSubtitle = CGFloat(0.58)
        static let progressBarTrack = CGFloat(0.38)
        static let hairline = CGFloat(0.05)
        static let subtleBorder = CGFloat(0.08)
        static let mediumBorder = CGFloat(0.16)
        static let dimText = CGFloat(0.48)
        static let mutedText = CGFloat(0.56)
        static let upNextBadge = CGFloat(0.72)
        static let coverEmoji = CGFloat(0.82)
        static let coverFallbackText = CGFloat(0.2)
        static let secondaryText = CGFloat(0.5)
        static let faintText = CGFloat(0.4)
        static let trackInactive = CGFloat(0.14)
        static let mutedLabel = CGFloat(0.45)
        static let valueText = CGFloat(0.55)
        static let placeholderText = CGFloat(0.3)
        static let borderFaint = CGFloat(0.15)
        static let iconMuted = CGFloat(0.6)
        static let playerTopScrim = CGFloat(0.72)
        static let playerBottomScrim = CGFloat(0.86)
        static let playerChromeDim = CGFloat(0.35)
        static let playerTextSecondary = CGFloat(0.7)
        static let playerTextDisabled = CGFloat(0.35)
        static let playerDeckDivider = CGFloat(0.08)
    }
  }
  enum Mobile {
    enum Dp {
        static let episodeCardWidth = CGFloat(244)
        static let episodeCardHeight = CGFloat(148)
        static let posterCardWidth = CGFloat(136)
        static let posterCardHeight = CGFloat(204)
        static let horizontalCardBase = CGFloat(196)
    }
  }
  enum Desktop {
    enum Dp {
        static let episodeCardWidth = CGFloat(300)
        static let episodeCardHeight = CGFloat(180)
        static let posterCardWidth = CGFloat(160)
        static let posterCardHeight = CGFloat(240)
        static let horizontalCardBase = CGFloat(220)
    }
  }
  enum Tv {
    enum Dp {
        static let episodeCardWidth = CGFloat(356)
        static let episodeCardHeight = CGFloat(208)
        static let posterCardWidth = CGFloat(136)
        static let posterCardHeight = CGFloat(204)
        static let horizontalCardBase = CGFloat(260)
    }
  }
  enum Compact {
    enum Dp {
        static let breakpointMaxWidth = CGFloat(600)
        static let pageHorizontalPadding = CGFloat(16)
        static let headerVerticalPadding = CGFloat(12)
        static let minimumCardWidth = CGFloat(150)
        static let horizontalSpacing = CGFloat(12)
        static let verticalSpacing = CGFloat(12)
        static let navigationHorizontalPadding = CGFloat(20)
        static let navigationVerticalPadding = CGFloat(10)
        static let navigationItemSpacing = CGFloat(6)
        static let navigationIconSize = CGFloat(24)
        static let navigationItemHorizontalPadding = CGFloat(12)
        static let navigationItemVerticalPadding = CGFloat(8)
        static let horizontalCardBase = CGFloat(196)
    }
    enum Sp {
        static let catalogTitleSize = CGFloat(24)
        static let navigationLabelSize = CGFloat(13)
    }
    enum Int {
        static let gridColumns = 3
    }
  }
  enum Medium {
    enum Dp {
        static let breakpointMaxWidth = CGFloat(840)
        static let pageHorizontalPadding = CGFloat(28)
        static let headerVerticalPadding = CGFloat(20)
        static let minimumCardWidth = CGFloat(160)
        static let horizontalSpacing = CGFloat(16)
        static let verticalSpacing = CGFloat(20)
        static let navigationHorizontalPadding = CGFloat(32)
        static let navigationVerticalPadding = CGFloat(12)
        static let navigationItemSpacing = CGFloat(8)
        static let navigationIconSize = CGFloat(26)
        static let navigationItemHorizontalPadding = CGFloat(14)
        static let navigationItemVerticalPadding = CGFloat(9)
        static let horizontalCardBase = CGFloat(210)
    }
    enum Sp {
        static let catalogTitleSize = CGFloat(26)
        static let navigationLabelSize = CGFloat(14)
    }
    enum Int {
        static let gridColumns = 5
    }
  }
  enum Expanded {
    enum Dp {
        static let pageHorizontalPadding = CGFloat(48)
        static let headerVerticalPadding = CGFloat(28)
        static let minimumCardWidth = CGFloat(170)
        static let horizontalSpacing = CGFloat(20)
        static let verticalSpacing = CGFloat(24)
        static let navigationHorizontalPadding = CGFloat(48)
        static let navigationVerticalPadding = CGFloat(16)
        static let navigationItemSpacing = CGFloat(12)
        static let navigationIconSize = CGFloat(28)
        static let navigationItemHorizontalPadding = CGFloat(16)
        static let navigationItemVerticalPadding = CGFloat(10)
        static let horizontalCardBase = CGFloat(260)
    }
    enum Sp {
        static let catalogTitleSize = CGFloat(28)
        static let navigationLabelSize = CGFloat(15)
    }
    enum Int {
        static let gridColumns = 7
    }
  }
  enum MaterialTheme {
    enum Dp {
        static let shapeExtraSmall = CGFloat(6)
        static let shapeSmall = CGFloat(10)
        static let shapeMedium = CGFloat(16)
        static let shapeLarge = CGFloat(24)
    }
    enum Sp {
        static let displayLarge = CGFloat(52)
        static let displayLargeLineHeight = CGFloat(56)
        static let displayLargeLetterSpacing = CGFloat(-1.5)
        static let displayMedium = CGFloat(40)
        static let displayMediumLineHeight = CGFloat(44)
        static let displayMediumLetterSpacing = CGFloat(-1)
        static let titleLarge = CGFloat(24)
        static let titleLargeLineHeight = CGFloat(28)
        static let titleMedium = CGFloat(18)
        static let titleMediumLineHeight = CGFloat(24)
        static let bodyLarge = CGFloat(17)
        static let bodyLargeLineHeight = CGFloat(26)
        static let bodyMedium = CGFloat(15)
        static let bodyMediumLineHeight = CGFloat(22)
        static let labelLarge = CGFloat(13)
        static let labelLargeLineHeight = CGFloat(16)
        static let labelLargeLetterSpacing = CGFloat(0.4)
    }
  }
}
