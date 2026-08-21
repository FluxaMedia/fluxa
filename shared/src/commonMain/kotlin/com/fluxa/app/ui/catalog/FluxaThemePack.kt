package com.fluxa.app.ui.catalog

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class FluxaThemeColors(
    val background: String,
    val backgroundElevated: String,
    val surface: String,
    val surfaceRaised: String,
    val navigation: String,
    val textPrimary: String,
    val textSecondary: String,
    val textMuted: String,
    val border: String,
    val borderStrong: String,
    val accent: String,
    val accentForeground: String,
    val success: String,
    val warning: String,
    val error: String,
    val info: String,
    val focus: String,
    val scrim: String,
)

@Serializable
data class FluxaThemeTypography(
    val displayFont: String,
    val bodyFont: String,
    val titleWeight: Int,
    val bodyWeight: Int,
)

@Serializable
data class FluxaThemeShape(
    val cardRadius: Int,
    val controlRadius: Int,
    val dialogRadius: Int,
)

@Serializable
data class FluxaThemeSpacing(
    val screenPadding: Int,
    val sectionGap: Int,
    val controlGap: Int,
)

@Serializable
data class FluxaThemeMotion(
    val enabled: Boolean,
    val fastMs: Int,
    val normalMs: Int,
    val slowMs: Int,
)

@Serializable
data class FluxaThemeLayouts(
    val home: String,
    val detail: String,
    val library: String,
    val navigation: String,
)

@Serializable
data class FluxaThemePack(
    val schemaVersion: Int,
    val id: String,
    val nameKey: String,
    val colors: FluxaThemeColors,
    val typography: FluxaThemeTypography,
    val shape: FluxaThemeShape,
    val spacing: FluxaThemeSpacing,
    val motion: FluxaThemeMotion,
    val layouts: FluxaThemeLayouts,
)

object FluxaThemePacks {
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

    private val json = Json { ignoreUnknownKeys = true }

    fun fromJson(raw: String): FluxaThemePack = runCatching { json.decodeFromString<FluxaThemePack>(raw) }.getOrDefault(fluxaDark)
}

fun FluxaThemePack.toColorScheme(accentOverride: Color? = null): ColorScheme {
    val theme = colors
    val accent = accentOverride ?: theme.accent.toThemeColor(FluxaColors.accent)
    return darkColorScheme(
        background = theme.background.toThemeColor(FluxaColors.background),
        surface = theme.surface.toThemeColor(FluxaColors.surface),
        surfaceVariant = theme.surfaceRaised.toThemeColor(FluxaColors.surfaceRaised),
        primary = accent,
        secondary = theme.textSecondary.toThemeColor(FluxaColors.textPrimary),
        onBackground = theme.textPrimary.toThemeColor(FluxaColors.textPrimary),
        onSurface = theme.textPrimary.toThemeColor(FluxaColors.textPrimary),
        onSurfaceVariant = theme.textSecondary.toThemeColor(FluxaColors.textPrimary),
        onPrimary = theme.accentForeground.toThemeColor(Color.Black),
        onSecondary = theme.accentForeground.toThemeColor(Color.Black),
        outline = theme.borderStrong.toThemeColor(FluxaColors.textPrimary.copy(alpha = 0.3f)),
        error = theme.error.toThemeColor(FluxaColors.errorRed),
    )
}

private fun String.toThemeColor(fallback: Color): Color {
    val value = removePrefix("#")
    val argb = when (value.length) {
        6 -> "FF$value"
        8 -> "${value.substring(6)}${value.substring(0, 6)}"
        else -> return fallback
    }
    return runCatching { Color(argb.toLong(16)) }.getOrDefault(fallback)
}
