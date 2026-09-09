@file:OptIn(androidx.compose.ui.text.ExperimentalTextApi::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import com.fluxa.app.R

val FluxaDisplay = FontFamily(
    Font(R.font.archivo, weight = FontWeight.Medium, variationSettings = FontVariation.Settings(FontVariation.weight(500), FontVariation.width(106f))),
    Font(R.font.archivo, weight = FontWeight.SemiBold, variationSettings = FontVariation.Settings(FontVariation.weight(600), FontVariation.width(112f))),
    Font(R.font.archivo, weight = FontWeight.Bold, variationSettings = FontVariation.Settings(FontVariation.weight(700), FontVariation.width(116f))),
    Font(R.font.archivo, weight = FontWeight.ExtraBold, variationSettings = FontVariation.Settings(FontVariation.weight(800), FontVariation.width(120f))),
    Font(R.font.archivo, weight = FontWeight.Black, variationSettings = FontVariation.Settings(FontVariation.weight(900), FontVariation.width(125f)))
)

private val DarkColorScheme = darkColorScheme(
    primary = Color.White,
    secondary = Color(0xFF7A8799),
    tertiary = FluxaColors.accentGold,
    background = FluxaColors.background,
    surface = FluxaColors.surface,
    surfaceVariant = FluxaColors.surfaceRaised,
    onPrimary = FluxaColors.background,
    onSecondary = FluxaColors.textPrimary,
    onTertiary = FluxaColors.background,
    onBackground = FluxaColors.textPrimary,
    onSurface = FluxaColors.textPrimary
)

private val AppTypography = Typography(
    displayLarge = TextStyle(
        fontFamily = FluxaDisplay,
        fontWeight = FontWeight.Black,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.displayLarge,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.displayLargeLineHeight,
        letterSpacing = FluxaUiLayoutTokens.MaterialTheme.Sp.displayLargeLetterSpacing
    ),
    displayMedium = TextStyle(
        fontFamily = FluxaDisplay,
        fontWeight = FontWeight.ExtraBold,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.displayMedium,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.displayMediumLineHeight,
        letterSpacing = FluxaUiLayoutTokens.MaterialTheme.Sp.displayMediumLetterSpacing
    ),
    titleLarge = TextStyle(
        fontFamily = FluxaDisplay,
        fontWeight = FontWeight.Bold,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.titleLarge,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.titleLargeLineHeight
    ),
    titleMedium = TextStyle(
        fontFamily = FluxaDisplay,
        fontWeight = FontWeight.SemiBold,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.titleMedium,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.titleMediumLineHeight
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Medium,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.bodyLarge,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.bodyLargeLineHeight
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Normal,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.bodyMedium,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.bodyMediumLineHeight
    ),
    labelLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Bold,
        fontSize = FluxaUiLayoutTokens.MaterialTheme.Sp.labelLarge,
        lineHeight = FluxaUiLayoutTokens.MaterialTheme.Sp.labelLargeLineHeight,
        letterSpacing = FluxaUiLayoutTokens.MaterialTheme.Sp.labelLargeLetterSpacing
    )
)

private val AppShapes = Shapes(
    extraSmall = androidx.compose.foundation.shape.RoundedCornerShape(FluxaUiLayoutTokens.MaterialTheme.Dp.shapeExtraSmall),
    small = androidx.compose.foundation.shape.RoundedCornerShape(FluxaUiLayoutTokens.MaterialTheme.Dp.shapeSmall),
    medium = androidx.compose.foundation.shape.RoundedCornerShape(FluxaUiLayoutTokens.MaterialTheme.Dp.shapeMedium),
    large = androidx.compose.foundation.shape.RoundedCornerShape(FluxaUiLayoutTokens.MaterialTheme.Dp.shapeLarge)
)

@Composable
fun AppTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        typography = AppTypography,
        shapes = AppShapes,
        content = content
    )
}
