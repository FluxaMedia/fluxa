package com.fluxa.app.ui.catalog

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.runtime.compositionLocalOf
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
    val fluxaDark = FluxaThemePackDefaults.fluxaDark

    private val json = Json { ignoreUnknownKeys = true }

    fun fromJson(raw: String): FluxaThemePack = runCatching { json.decodeFromString<FluxaThemePack>(raw) }.getOrDefault(fluxaDark)

    fun parseJson(raw: String): FluxaThemePack? {
        if (raw.encodeToByteArray().size > 256 * 1024) return null
        return runCatching { json.decodeFromString<FluxaThemePack>(raw) }
            .getOrNull()
            ?.takeIf { it.schemaVersion == 1 && it.id.matches(Regex("[a-z0-9][a-z0-9-]{0,63}")) }
    }
}

val LocalFluxaThemePack = compositionLocalOf { FluxaThemePacks.fluxaDark }

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
