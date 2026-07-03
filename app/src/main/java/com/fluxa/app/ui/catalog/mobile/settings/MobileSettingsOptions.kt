@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.annotation.DrawableRes
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.drawscope.Stroke
import coil3.compose.AsyncImage
import java.util.Locale

internal fun mobilePresetOptions(lang: String): List<ChoiceOption> {
    return listOf(
        ChoiceOption("small", AppStrings.t(lang, "auto.small")),
        ChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        ChoiceOption("large", AppStrings.t(lang, "auto.large"))
    )
}

internal fun mobilePresetLabel(value: String, lang: String): String {
    return when (value) {
        "small" -> AppStrings.t(lang, "auto.small")
        "large" -> AppStrings.t(lang, "auto.large")
        else -> AppStrings.t(lang, "auto.medium")
    }
}

internal fun mobileCornerRadius(value: String) = when (value) {
    "sharp" -> 2.dp
    "small" -> 6.dp
    "classic" -> 8.dp
    "soft" -> 12.dp
    "rounded" -> 18.dp
    "pill" -> 28.dp
    "large" -> 16.dp
    else -> 10.dp
}

internal fun mobilePosterCornerOptions(lang: String): List<ChoiceOption> {
    return listOf(
        ChoiceOption("sharp", AppStrings.t(lang, "auto.sharp")),
        ChoiceOption("classic", AppStrings.t(lang, "auto.classic")),
        ChoiceOption("soft", AppStrings.t(lang, "auto.soft")),
        ChoiceOption("rounded", AppStrings.t(lang, "auto.rounded")),
        ChoiceOption("pill", AppStrings.t(lang, "auto.extra_rounded"))
    )
}

internal fun mobilePosterCornerLabel(value: String, lang: String): String {
    return when (value) {
        "sharp" -> AppStrings.t(lang, "auto.sharp")
        "classic", "small" -> AppStrings.t(lang, "auto.classic")
        "rounded", "large" -> AppStrings.t(lang, "auto.rounded")
        "pill" -> AppStrings.t(lang, "auto.extra_rounded")
        else -> AppStrings.t(lang, "auto.soft")
    }
}

internal fun mobilePosterWidthOptions(lang: String): List<ChoiceOption> {
    return listOf(
        ChoiceOption("xsmall", AppStrings.t(lang, "auto.very_small")),
        ChoiceOption("small", AppStrings.t(lang, "auto.small")),
        ChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        ChoiceOption("large", AppStrings.t(lang, "auto.large")),
        ChoiceOption("xlarge", AppStrings.t(lang, "auto.very_large"))
    )
}

internal fun mobilePosterWidthLabel(value: String, lang: String): String {
    return when (value) {
        "xsmall" -> AppStrings.t(lang, "auto.very_small")
        "small" -> AppStrings.t(lang, "auto.small")
        "large" -> AppStrings.t(lang, "auto.large")
        "xlarge" -> AppStrings.t(lang, "auto.very_large")
        else -> AppStrings.t(lang, "auto.medium")
    }
}

internal fun mobilePosterWidth(value: String, landscape: Boolean): Dp {
    val base = if (landscape) 158.dp else 104.dp
    val delta = when (value) {
        "xsmall" -> (-18).dp
        "small" -> (-10).dp
        "large" -> 16.dp
        "xlarge" -> 30.dp
        else -> 0.dp
    }
    return base + delta
}

internal fun mobilePosterHeight(value: String, landscape: Boolean): Dp {
    val width = mobilePosterWidth(value, landscape)
    return if (landscape) width * 0.58f else width * 1.5f
}

internal fun mobileDensityPadding(value: String) = when (value) {
    "small" -> 13.dp
    "large" -> 20.dp
    else -> 16.dp
}

internal fun mobileStartPageLabel(value: String, lang: String): String {
    return when (value) {
        "discover" -> AppStrings.t(lang, "nav.discover")
        "library" -> AppStrings.t(lang, "nav.library")
        else -> AppStrings.t(lang, "nav.home")
    }
}

internal fun mobileSeasonSelectorOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("dropdown", AppStrings.t(lang, "settings.season_selector_dropdown")),
    ChoiceOption("tabs", AppStrings.t(lang, "settings.season_selector_tabs")),
    ChoiceOption("posters", AppStrings.t(lang, "settings.season_selector_posters"))
)

internal fun mobileSeasonSelectorLabel(value: String, lang: String): String {
    return mobileSeasonSelectorOptions(lang).firstOrNull { it.value == value }?.label
        ?: AppStrings.t(lang, "settings.season_selector_dropdown")
}

internal fun settingsContinueWatchingTitle(lang: String): String =
    AppStrings.t(lang, "auto.continue_watching").uppercase()

internal fun settingsPipTitle(lang: String): String =
    AppStrings.t(lang, "settings.pip_mode")

internal fun settingsSecondsLabel(lang: String, seconds: Int): String =
    AppStrings.format(lang, "format.seconds", seconds)

internal fun bufferCacheLabel(valueMb: Int): String =
    if (valueMb >= 1000) "${valueMb / 1000} GB" else "$valueMb MB"

internal fun bufferSecondOptions(lang: String, includeZero: Boolean = false): List<ChoiceOption> {
    val values = if (includeZero) listOf(0, 15, 30, 60, 120, 300) else listOf(30, 60, 120, 300, 600)
    return values.map { ChoiceOption(it.toString(), settingsSecondsLabel(lang, it)) }
}

internal fun minBufferSecondOptions(lang: String): List<ChoiceOption> =
    listOf(2, 4, 8, 15, 30).map { ChoiceOption(it.toString(), settingsSecondsLabel(lang, it)) }

internal fun bufferMsLabel(ms: Int): String =
    if (ms % 1000 == 0) "${ms / 1000} s" else "${"%.1f".format(ms / 1000.0)} s"

internal fun playbackBufferMsOptions(): List<ChoiceOption> =
    listOf(500, 1000, 1500, 2000, 3000, 5000).map { ChoiceOption(it.toString(), bufferMsLabel(it)) }

internal fun rebufferBufferMsOptions(): List<ChoiceOption> =
    listOf(1000, 1500, 2000, 3000, 5000, 8000).map { ChoiceOption(it.toString(), bufferMsLabel(it)) }

internal fun audioDecoderModeOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("hw_only", AppStrings.t(lang, "settings.audio_decoder_hw_only")),
    ChoiceOption("hw_prefer", AppStrings.t(lang, "settings.audio_decoder_hw_prefer")),
    ChoiceOption("sw_only", AppStrings.t(lang, "settings.audio_decoder_sw_only"))
)

internal fun audioDecoderModeLabel(value: String, lang: String): String {
    return audioDecoderModeOptions(lang).firstOrNull { it.value == value }?.label
        ?: AppStrings.t(lang, "settings.audio_decoder_hw_prefer")
}

internal fun dolbyVisionFallbackOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("auto", AppStrings.t(lang, "settings.dv_fallback_auto")),
    ChoiceOption("force_hdr10", AppStrings.t(lang, "settings.dv_fallback_force_hdr10")),
    ChoiceOption("off", AppStrings.t(lang, "settings.dv_fallback_off"))
)

internal fun dolbyVisionFallbackLabel(value: String, lang: String): String {
    return dolbyVisionFallbackOptions(lang).firstOrNull { it.value == value }?.label
        ?: AppStrings.t(lang, "settings.dv_fallback_auto")
}

internal fun dvRpuModeOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("2", AppStrings.t(lang, "settings.dv_rpu_mode_2")),
    ChoiceOption("1", AppStrings.t(lang, "settings.dv_rpu_mode_1")),
    ChoiceOption("4", AppStrings.t(lang, "settings.dv_rpu_mode_4"))
)

internal fun dvRpuModeLabel(value: Int, lang: String): String =
    dvRpuModeOptions(lang).firstOrNull { it.value == value.toString() }?.label
        ?: AppStrings.t(lang, "settings.dv_rpu_mode_2")

internal fun dvHdr10PlusModeOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("auto", AppStrings.t(lang, "settings.dv_hdr10plus_mode_auto")),
    ChoiceOption("always", AppStrings.t(lang, "settings.dv_hdr10plus_mode_always")),
    ChoiceOption("never", AppStrings.t(lang, "settings.dv_hdr10plus_mode_never"))
)

internal fun verticalPosterLabel(lang: String): String =
    AppStrings.t(lang, "settings.vertical_poster")

internal fun horizontalPosterLabel(lang: String): String =
    AppStrings.t(lang, "settings.horizontal_poster")

internal fun languageDisplayName(language: String, lang: String): String = when (language.lowercase()) {
    "none", "", "__off__" -> AppStrings.t(lang, "settings.none")
    "forced" -> AppStrings.t(lang, "settings.forced")
    "original" -> AppStrings.t(lang, "settings.original")
    "device_language" -> AppStrings.t(lang, "settings.device_language")
    "tr", "tr-tr" -> Locale.forLanguageTag("tr").getDisplayLanguage(Locale.forLanguageTag("tr"))
    "en", "en-us" -> "English"
    else -> Locale.forLanguageTag(language).getDisplayLanguage(Locale.forLanguageTag(language)).takeIf { it.isNotBlank() } ?: language
}

private val COMMON_LANGUAGE_CODES = listOf(
    "tr", "en", "ja", "ko", "zh", "es", "fr", "de", "it", "pt", "ru", "ar", "hi",
    "nl", "pl", "sv", "no", "da", "fi", "el", "he", "th", "vi", "id", "ms",
    "cs", "hu", "ro", "uk", "bg", "hr", "sk", "sr", "ca", "fa"
)

internal fun audioLanguageOptions(lang: String): List<ChoiceOption> {
    val special = listOf(
        ChoiceOption("none", AppStrings.t(lang, "settings.none")),
        ChoiceOption("original", AppStrings.t(lang, "settings.original")),
        ChoiceOption("device_language", AppStrings.t(lang, "settings.device_language"))
    )
    val languages = COMMON_LANGUAGE_CODES.mapNotNull { code ->
        val locale = Locale.forLanguageTag(code)
        val label = locale.getDisplayLanguage(locale).replaceFirstChar {
            if (it.isLowerCase()) it.titlecase(locale) else it.toString()
        }
        if (label.isBlank()) null else ChoiceOption(code, label)
    }
    return special + languages
}

internal fun subtitleLanguageOptions(lang: String): List<ChoiceOption> {
    val special = listOf(
        ChoiceOption("none", AppStrings.t(lang, "settings.none")),
        ChoiceOption("forced", AppStrings.t(lang, "settings.forced"))
    )
    val languages = COMMON_LANGUAGE_CODES.mapNotNull { code ->
        val locale = Locale.forLanguageTag(code)
        val label = locale.getDisplayLanguage(locale).replaceFirstChar {
            if (it.isLowerCase()) it.titlecase(locale) else it.toString()
        }
        if (label.isBlank()) null else ChoiceOption(code, label)
    }
    return special + languages
}

internal fun playerChoiceLabel(value: String, lang: String): String {
    return when (value) {
        "mpv" -> "MPV"
        else -> "ExoPlayer"
    }
}

internal fun subtitleSizePercentLabel(value: Float): String = "${value.toInt()}%"

internal fun mobileAutoplayLabel(value: String, lang: String): String {
    return when (value) {
        "off" -> AppStrings.t(lang, "common.off")
        else -> AppStrings.t(lang, "auto.next_episode")
    }
}

internal fun mobileDataUsageLabel(value: String, lang: String): String {
    return when (value) {
        "low" -> AppStrings.t(lang, "auto.low")
        "high" -> AppStrings.t(lang, "auto.high")
        else -> AppStrings.t(lang, "auto.medium")
    }
}

internal fun mobilePlaybackSpeedLabel(value: Float): String {
    return when (value) {
        0.75f -> "0.75x"
        1.25f -> "1.25x"
        1.5f -> "1.50x"
        else -> "1.00x"
    }
}

internal fun mobileColorOptions(lang: String): List<ChoiceOption> {
    return listOf(
        ChoiceOption(Color.White.toArgb().toString(), AppStrings.t(lang, "auto.white")),
        ChoiceOption(Color.Black.toArgb().toString(), AppStrings.t(lang, "auto.black")),
        ChoiceOption(Color(0xFFFFE45C).toArgb().toString(), AppStrings.t(lang, "auto.yellow")),
        ChoiceOption(Color(0xFFFF5D5D).toArgb().toString(), AppStrings.t(lang, "auto.red")),
        ChoiceOption(Color(0xFF3F7CFF).toArgb().toString(), AppStrings.t(lang, "auto.blue"))
    )
}

internal fun mobileSubtitleColorValues(): List<Int> {
    return listOf(
        Color.White.toArgb(),
        Color.Black.toArgb(),
        Color(0xFFFFE45C).toArgb(),
        Color(0xFFFF5D5D).toArgb(),
        Color(0xFF3F7CFF).toArgb(),
        Color(0xFF54D17A).toArgb(),
        Color(0xFFFF8A3D).toArgb(),
        Color(0xFFC084FC).toArgb()
    )
}

internal fun mobileSubtitleColorOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption(Color.White.toArgb().toString(), AppStrings.t(lang, "auto.white")),
    ChoiceOption(Color.Black.toArgb().toString(), AppStrings.t(lang, "auto.black")),
    ChoiceOption(Color(0xFFFFE45C).toArgb().toString(), AppStrings.t(lang, "auto.yellow")),
    ChoiceOption(Color(0xFFFF5D5D).toArgb().toString(), AppStrings.t(lang, "auto.red")),
    ChoiceOption(Color(0xFF3F7CFF).toArgb().toString(), AppStrings.t(lang, "auto.blue")),
    ChoiceOption(Color(0xFF54D17A).toArgb().toString(), AppStrings.t(lang, "auto.green")),
    ChoiceOption(Color(0xFFFF8A3D).toArgb().toString(), AppStrings.t(lang, "auto.orange")),
    ChoiceOption(Color(0xFFC084FC).toArgb().toString(), AppStrings.t(lang, "auto.purple"))
)

internal fun mobileColorLabel(value: Int, lang: String): String {
    return when (value) {
        Color.White.toArgb() -> AppStrings.t(lang, "auto.white")
        Color.Black.toArgb() -> AppStrings.t(lang, "auto.black")
        Color(0xFFFFE45C).toArgb() -> AppStrings.t(lang, "auto.yellow")
        Color(0xFFFF5D5D).toArgb() -> AppStrings.t(lang, "auto.red")
        Color(0xFF3F7CFF).toArgb() -> AppStrings.t(lang, "auto.blue")
        Color(0xFF54D17A).toArgb() -> AppStrings.t(lang, "auto.green")
        Color(0xFFFF8A3D).toArgb() -> AppStrings.t(lang, "auto.orange")
        Color(0xFFC084FC).toArgb() -> AppStrings.t(lang, "auto.purple")
        else -> "#%06X".format(value and 0x00FFFFFF)
    }
}

internal fun mobileOpacityOptions(): List<ChoiceOption> {
    return listOf(
        ChoiceOption("1.0", "100%"),
        ChoiceOption("0.75", "75%"),
        ChoiceOption("0.5", "50%"),
        ChoiceOption("0.25", "25%"),
        ChoiceOption("0.0", "0%")
    )
}

internal fun mobileOpacityLabel(value: Float): String {
    return "${(value.coerceIn(0f, 1f) * 100).toInt()}%"
}
