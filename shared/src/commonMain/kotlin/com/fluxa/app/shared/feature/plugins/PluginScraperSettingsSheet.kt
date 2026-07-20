package com.fluxa.app.shared.feature.plugins

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PluginScraperSettingsSheet(
    scraperName: String,
    language: String?,
    loading: Boolean,
    fields: List<PluginSettingsFieldUiModel>,
    initialValues: Map<String, Any?>,
    onDismiss: () -> Unit,
    onSave: (Map<String, Any?>) -> Unit
) {
    val values = remember(fields, initialValues) {
        val map = mutableStateMapOf<String, Any?>()
        fields.forEach { field ->
            if (field.key.isEmpty()) return@forEach
            map[field.key] = initialValues[field.key] ?: when (field.type) {
                "toggle" -> field.defaultBoolean
                else -> field.defaultValue
            }
        }
        map
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = Color(0xFF111318),
        contentColor = Color.White
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 22.dp, vertical = 10.dp)
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom))
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Text(
                AppStrings.format(language, "settings.plugins.settings_title", scraperName),
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.Black
            )

            when {
                loading -> {
                    Box(modifier = Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator(color = Color.White, strokeWidth = 2.dp, modifier = Modifier.size(24.dp))
                    }
                }
                fields.isEmpty() -> {
                    Text(
                        AppStrings.t(language, "settings.plugins.settings_empty"),
                        color = Color.White.copy(alpha = 0.5f),
                        modifier = Modifier.padding(vertical = 12.dp)
                    )
                }
                else -> {
                    fields.forEach { field ->
                        PluginSettingsField(
                            field = field,
                            language = language,
                            value = values[field.key],
                            onValueChange = { values[field.key] = it }
                        )
                    }
                }
            }

            if (!loading && fields.isNotEmpty()) {
                Button(
                    onClick = { onSave(values.toMap()) },
                    modifier = Modifier.fillMaxWidth().height(52.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                    shape = RoundedCornerShape(12.dp)
                ) {
                    Text(AppStrings.t(language, "settings.plugins.settings_save"), fontWeight = FontWeight.Black)
                }
            }
        }
    }
}

@Composable
private fun PluginSettingsField(
    field: PluginSettingsFieldUiModel,
    language: String?,
    value: Any?,
    onValueChange: (Any?) -> Unit
) {
    when (field.type) {
        "header" -> {
            Text(
                field.label,
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
        "info" -> {
            Text(
                field.label,
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 13.sp
            )
        }
        "text" -> {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(field.label, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                TextField(
                    value = value as? String ?: "",
                    onValueChange = onValueChange,
                    placeholder = { field.placeholder?.let { Text(it, color = Color.White.copy(alpha = 0.35f)) } },
                    singleLine = true,
                    visualTransformation = if (field.isPassword) PasswordVisualTransformation() else VisualTransformation.None,
                    keyboardOptions = KeyboardOptions(keyboardType = if (field.isPassword) KeyboardType.Password else KeyboardType.Text),
                    modifier = Modifier.fillMaxWidth(),
                    colors = TextFieldDefaults.colors(
                        focusedContainerColor = Color.White.copy(alpha = 0.05f),
                        unfocusedContainerColor = Color.White.copy(alpha = 0.05f),
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White
                    )
                )
                field.description?.let {
                    Text(it, color = Color.White.copy(alpha = 0.4f), fontSize = 11.sp)
                }
            }
        }
        "select" -> {
            var expanded by remember { mutableStateOf(false) }
            val currentValue = value as? String ?: field.defaultValue ?: ""
            val selectedLabel = field.options.find { it.value == currentValue }?.label
                ?: currentValue.ifBlank { AppStrings.t(language, "settings.plugins.select_placeholder") }

            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(field.label, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                Box {
                    OutlinedButton(
                        onClick = { expanded = true },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        contentPadding = PaddingValues(16.dp)
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(selectedLabel, color = Color.White)
                            Icon(Icons.Rounded.ArrowDropDown, contentDescription = null, tint = Color.White.copy(alpha = 0.6f))
                        }
                    }
                    DropdownMenu(
                        expanded = expanded,
                        onDismissRequest = { expanded = false },
                        modifier = Modifier.fillMaxWidth(0.85f)
                    ) {
                        field.options.forEach { option ->
                            DropdownMenuItem(
                                text = { Text(option.label) },
                                onClick = {
                                    onValueChange(option.value)
                                    expanded = false
                                }
                            )
                        }
                    }
                }
                field.description?.let {
                    Text(it, color = Color.White.copy(alpha = 0.4f), fontSize = 11.sp)
                }
            }
        }
        "toggle" -> {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(field.label, color = Color.White, fontSize = 14.sp)
                    field.description?.let {
                        Text(it, color = Color.White.copy(alpha = 0.4f), fontSize = 11.sp)
                    }
                }
                Switch(
                    checked = value as? Boolean ?: field.defaultBoolean,
                    onCheckedChange = onValueChange,
                    colors = SwitchDefaults.colors(
                        checkedTrackColor = Color.White,
                        checkedThumbColor = Color.Black,
                        uncheckedTrackColor = Color.White.copy(alpha = 0.15f),
                        uncheckedThumbColor = Color.White.copy(alpha = 0.7f)
                    )
                )
            }
        }
    }
}
