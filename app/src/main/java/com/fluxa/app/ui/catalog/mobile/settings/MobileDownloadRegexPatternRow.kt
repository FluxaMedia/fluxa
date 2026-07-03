@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
internal fun MobileDownloadRegexPatternRow(
    lang: String,
    value: String,
    onValueChange: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text(
            text = AppStrings.t(lang, "settings.regex_pattern"),
            color = colors.text.copy(alpha = 0.9f),
            fontWeight = FontWeight.Bold
        )
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            placeholder = { Text(AppStrings.t(lang, "settings.regex_pattern_placeholder")) },
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = colors.text.copy(alpha = 0.72f),
                unfocusedBorderColor = colors.text.copy(alpha = 0.18f),
                focusedTextColor = colors.text,
                unfocusedTextColor = colors.text,
                cursorColor = colors.text,
                focusedPlaceholderColor = colors.text.copy(alpha = 0.34f),
                unfocusedPlaceholderColor = colors.text.copy(alpha = 0.28f),
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent
            )
        )
    }
}
