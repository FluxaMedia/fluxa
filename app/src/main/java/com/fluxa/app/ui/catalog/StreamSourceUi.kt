package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

internal fun Stream.streamSourceHeader(): String {
    return name
        ?.takeIf { it.isNotBlank() }
        ?: title?.takeIf { it.isNotBlank() }
        ?: description?.takeIf { it.isNotBlank() }
        ?: addonName?.takeIf { it.isNotBlank() }
        ?: playableUrl.orEmpty()
}

internal fun Stream.streamRawBody(): String? {
    val header = streamSourceHeader()
    return listOf(description, title, addonName)
        .firstOrNull { value -> !value.isNullOrBlank() && value != header }
}

internal fun Stream.cloudstreamPlaybackDetailLine(): String? {
    val addon = addonName?.trim().orEmpty()
    if (addon.isBlank()) return null

    val quality = name
        ?.lines()
        ?.map { it.trim() }
        ?.firstOrNull { line -> line.isNotBlank() && line != addon }
        ?: return null

    val fileName = title?.trim()?.takeIf { it.isNotBlank() }
        ?: effectiveFilename?.trim()?.takeIf { it.isNotBlank() }
        ?: url?.substringBefore('?')?.substringAfterLast('/')?.trim()?.takeIf { it.isNotBlank() }
        ?: return null
    return listOf(fileName, quality).joinToString(" - ")
}

@Composable
internal fun AddonStreamBodyText(
    text: String,
    modifier: Modifier = Modifier,
    bodyMaxLines: Int = 6,
    contentColor: Color = Color.White.copy(alpha = 0.78f)
) {
    val lines = text.lines().map { it.trim() }.filter { it.isNotBlank() }
    if (lines.size <= 1) {
        Text(
            text = text,
            color = contentColor,
            fontSize = 10.5.sp,
            lineHeight = 15.sp,
            maxLines = bodyMaxLines + 1,
            overflow = TextOverflow.Ellipsis,
            modifier = modifier
        )
        return
    }

    val body = lines.dropLast(1).joinToString("\n")
    val footer = lines.last()
    Column(modifier = modifier.fillMaxWidth()) {
        Text(
            text = body,
            color = contentColor,
            fontSize = 10.5.sp,
            lineHeight = 15.sp,
            maxLines = bodyMaxLines,
            overflow = TextOverflow.Ellipsis
        )
        Text(
            text = footer,
            color = Color.White.copy(alpha = 0.86f),
            fontSize = 10.5.sp,
            lineHeight = 14.sp,
            fontWeight = FontWeight.Medium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = 5.dp)
        )
    }
}
