package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Stream

internal const val STREAM_SOURCE_MODE_MANUAL = "manual"
internal const val STREAM_SOURCE_MODE_FIRST = "first"
internal const val STREAM_SOURCE_MODE_REGEX = "regex"

internal fun streamSourceSelectionOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption(STREAM_SOURCE_MODE_MANUAL, AppStrings.t(lang, "settings.stream_source_manual")),
    ChoiceOption(STREAM_SOURCE_MODE_FIRST, AppStrings.t(lang, "settings.stream_source_first")),
    ChoiceOption(STREAM_SOURCE_MODE_REGEX, AppStrings.t(lang, "settings.stream_source_regex"))
)

internal fun streamSourceSelectionLabel(value: String, lang: String): String {
    return streamSourceSelectionOptions(lang).firstOrNull { it.value == value }?.label
        ?: AppStrings.t(lang, "settings.stream_source_manual")
}

internal fun downloadSubtitleOptions(lang: String): List<ChoiceOption> = listOf(
    ChoiceOption("preferred", AppStrings.t(lang, "settings.download_subtitle_preferred")),
    ChoiceOption("off", AppStrings.t(lang, "settings.download_subtitle_off")),
    ChoiceOption("tr", languageDisplayName("tr", lang)),
    ChoiceOption("en", languageDisplayName("en", lang)),
    ChoiceOption("ja", languageDisplayName("ja", lang)),
    ChoiceOption("es", languageDisplayName("es", lang)),
    ChoiceOption("fr", languageDisplayName("fr", lang)),
    ChoiceOption("de", languageDisplayName("de", lang))
)

internal fun downloadSubtitleLabel(value: String, lang: String): String {
    return downloadSubtitleOptions(lang).firstOrNull { it.value == value }?.label
        ?: AppStrings.t(lang, "settings.download_subtitle_preferred")
}

internal fun selectStreamIndex(
    streams: List<Stream>,
    currentVideoId: String?,
    initialStreamIndex: Int,
    savedUrl: String?,
    savedTitle: String?,
    sourceSelectionMode: String,
    regexPattern: String?,
    preferredBingeGroup: String?
): Int {
    return FluxaCoreNative.selectStreamIndex(
        streams = streams,
        currentVideoId = currentVideoId,
        initialStreamIndex = initialStreamIndex,
        savedUrl = savedUrl,
        savedTitle = savedTitle,
        sourceSelectionMode = sourceSelectionMode,
        regexPattern = regexPattern,
        preferredBingeGroup = preferredBingeGroup
    )
}
