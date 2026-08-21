package com.fluxa.app.ui.catalog

object FluxaThemePackDefaults {
    val fluxaDark = FluxaThemePack(
        schemaVersion = 1,
        id = "fluxa-dark",
        nameKey = "theme.fluxa_dark",
        colors = FluxaThemeColors(
            background = "#060606",
            backgroundElevated = "#0A0A0B",
            surface = "#141416",
            surfaceRaised = "#1C1C1F",
            navigation = "#0D0D0D",
            textPrimary = "#FFFFFF",
            textSecondary = "#B8B8BE",
            textMuted = "#8A8A91",
            border = "#FFFFFF14",
            borderStrong = "#FFFFFF29",
            accent = "#E85D3F",
            accentForeground = "#FFFFFF",
            success = "#45D483",
            warning = "#F0C674",
            error = "#FF6B6B",
            info = "#2196F3",
            focus = "#FFFFFF",
            scrim = "#000000C7",
        ),
        typography = FluxaThemeTypography("Archivo", "Montserrat", 700, 400),
        shape = FluxaThemeShape(12, 8, 16),
        spacing = FluxaThemeSpacing(24, 20, 12),
        motion = FluxaThemeMotion(true, 120, 180, 300),
        layouts = FluxaThemeLayouts("shelves", "hero-with-rail", "poster-grid", "sidebar"),
    )
}
