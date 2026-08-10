package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions

@Composable
fun SettingsActionRow(
    label: String,
    value: String? = null,
    destructive: Boolean = false,
    icon: (@Composable () -> Unit)? = null,
    onClick: () -> Unit
) {
    val highlighted = LocalSettingsHighlightLabel.current == label
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    LaunchedEffect(highlighted) {
        if (highlighted) bringIntoViewRequester.bringIntoView()
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .bringIntoViewRequester(bringIntoViewRequester)
            .settingsHighlight(highlighted)
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable(onClick = onClick)
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (icon != null) {
                Box(modifier = Modifier.size(22.dp), contentAlignment = Alignment.Center) { icon() }
                Spacer(Modifier.width(14.dp))
            }
            Text(label, color = if (destructive) FluxaColors.errorRed else Color.White, style = MaterialTheme.typography.bodyMedium)
        }
        if (value != null) {
            Text(
                value,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
            )
        }
    }
}

@Composable
fun SettingsConnectionRow(
    label: String,
    connected: Boolean,
    connectedLabel: String,
    icon: (@Composable () -> Unit)? = null,
    hasSyncFailure: Boolean = false,
    syncFailedLabel: String? = null,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().settingsRowDivider().settingsFocusRing().clickable(onClick = onClick).padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (icon != null) {
                Box(modifier = Modifier.size(28.dp), contentAlignment = Alignment.Center) {
                    Box(modifier = Modifier.scale(28f / 34f)) {
                        icon()
                    }
                }
            } else {
                Box(
                    modifier = Modifier
                        .size(28.dp)
                        .clip(RoundedCornerShape(FluxaDimensions.CornerPresets.classic))
                        .background(Color.White.copy(alpha = FluxaDimensions.Alpha.subtleBorder)),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = label.take(1).uppercase(),
                        color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                        style = MaterialTheme.typography.bodySmall,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
            Spacer(Modifier.width(14.dp))
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            val statusText = if (connected && hasSyncFailure) syncFailedLabel.orEmpty() else if (connected) connectedLabel else null
            if (statusText != null) {
                Text(
                    statusText,
                    color = if (hasSyncFailure) FluxaColors.errorRed else Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 140.dp)
                )
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
    }
}

@Composable
fun SettingsInlineSecretField(
    label: String,
    value: String,
    placeholder: String? = null,
    onValueChanged: (String) -> Unit
) {
    var revealed by remember { mutableStateOf(false) }
    var text by remember(value) { mutableStateOf(value) }
    LaunchedEffect(text) {
        if (text != value) {
            kotlinx.coroutines.delay(500)
            onValueChanged(text)
        }
    }
    androidx.compose.material3.OutlinedTextField(
        value = text,
        onValueChange = { text = it },
        label = { Text(label) },
        placeholder = placeholder?.let { { Text(it, color = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText)) } },
        singleLine = true,
        visualTransformation = if (revealed) VisualTransformation.None else PasswordVisualTransformation(),
        trailingIcon = {
            IconButtonToggle(revealed) { revealed = !revealed }
        },
        modifier = Modifier.fillMaxWidth(),
        colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
            focusedTextColor = Color.White,
            unfocusedTextColor = Color.White,
            focusedBorderColor = LocalSettingsAccentColor.current,
            unfocusedBorderColor = Color.White.copy(alpha = FluxaDimensions.Alpha.borderFaint),
            focusedLabelColor = LocalSettingsAccentColor.current,
            unfocusedLabelColor = Color.White.copy(alpha = FluxaDimensions.Alpha.faintText),
            cursorColor = LocalSettingsAccentColor.current
        )
    )
}

@Composable
fun SettingsSecretFieldRow(
    label: String,
    value: String,
    placeholder: String? = null,
    onValueChanged: (String) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable { showDialog = true }
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                if (value.isBlank()) placeholder.orEmpty() else "••••••••",
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
            )
            Spacer(Modifier.width(4.dp))
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
    }
    if (showDialog) {
        SettingsTextEditDialog(
            title = label,
            value = value,
            placeholder = placeholder,
            isSecret = true,
            onValueChanged = onValueChanged,
            onDismiss = { showDialog = false }
        )
    }
}

@Composable
private fun SettingsTextEditDialog(
    title: String,
    value: String,
    placeholder: String? = null,
    isSecret: Boolean = false,
    onValueChanged: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var text by remember { mutableStateOf(value) }
    var revealed by remember { mutableStateOf(false) }
    androidx.compose.ui.window.Dialog(onDismissRequest = { onValueChanged(text); onDismiss() }) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(20.dp))
                .background(FluxaColors.surfaceRaised)
                .padding(vertical = 20.dp, horizontal = 20.dp)
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = Color.White)
            Spacer(Modifier.height(16.dp))
            androidx.compose.material3.OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                placeholder = placeholder?.let { { Text(it, color = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText)) } },
                singleLine = true,
                visualTransformation = if (isSecret && !revealed) PasswordVisualTransformation() else VisualTransformation.None,
                trailingIcon = if (isSecret) {
                    { IconButtonToggle(revealed) { revealed = !revealed } }
                } else null,
                modifier = Modifier.fillMaxWidth(),
                colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    focusedBorderColor = LocalSettingsAccentColor.current,
                    unfocusedBorderColor = Color.White.copy(alpha = FluxaDimensions.Alpha.borderFaint),
                    cursorColor = LocalSettingsAccentColor.current
                )
            )
        }
    }
}

@Composable
private fun IconButtonToggle(revealed: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier.size(24.dp).settingsFocusRing(shape = CircleShape).clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = if (revealed) Icons.Filled.VisibilityOff else Icons.Filled.Visibility,
            contentDescription = null,
            tint = Color.White.copy(alpha = FluxaDimensions.Alpha.iconMuted),
            modifier = Modifier.size(20.dp)
        )
    }
}

@Composable
fun SettingsInfoRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth().settingsRowDivider().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(label, color = Color.White.copy(alpha = FluxaDimensions.Alpha.iconMuted), style = MaterialTheme.typography.bodyMedium)
        Text(value, color = Color.White.copy(alpha = 0.85f), style = MaterialTheme.typography.bodySmall)
    }
}

@Composable
fun SettingsConnectedAccountCard(statusLabel: String, email: String, badgeText: String) {
    SettingsGroupCard {
        Row(
            modifier = Modifier.fillMaxWidth().padding(vertical = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.Top
        ) {
            Column {
                Text(statusLabel, color = Color.White.copy(alpha = FluxaDimensions.Alpha.iconMuted), style = MaterialTheme.typography.bodySmall)
                Spacer(Modifier.height(4.dp))
                Text(email, color = Color.White, style = MaterialTheme.typography.bodyLarge)
            }
            Text(
                badgeText.uppercase(),
                color = FluxaColors.successGreen,
                fontWeight = FontWeight.Bold,
                fontSize = 11.sp,
                letterSpacing = 0.6.sp
            )
        }
    }
}

@Composable
fun SettingsPrimaryButton(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(FluxaDimensions.CornerPresets.pill))
            .background(Color.White)
            .clickable(onClick = onClick)
            .padding(vertical = 16.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(label, color = Color.Black, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
fun SettingsDestructiveLink(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(label, color = FluxaColors.errorRed, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
fun SettingsNavRow(
    label: String,
    description: String? = null,
    value: String? = null,
    onClick: () -> Unit
) {
    val highlighted = LocalSettingsHighlightLabel.current == label
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    LaunchedEffect(highlighted) {
        if (highlighted) bringIntoViewRequester.bringIntoView()
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .bringIntoViewRequester(bringIntoViewRequester)
            .settingsHighlight(highlighted)
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable(onClick = onClick)
            .padding(vertical = if (description != null) 10.dp else 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.weight(1f)) {
            Column {
                Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
                if (description != null) {
                    Text(description, color = Color.White.copy(alpha = FluxaDimensions.Alpha.mutedLabel), style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 2.dp))
                }
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (value != null) {
                Text(
                    value,
                    color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                    fontSize = 13.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 140.dp)
                )
                Spacer(Modifier.width(8.dp))
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
fun SettingsTextFieldRow(
    label: String,
    value: String,
    onValueChanged: (String) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable { showDialog = true }
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                value,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
            )
            Spacer(Modifier.width(4.dp))
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
    }
    if (showDialog) {
        SettingsTextEditDialog(
            title = label,
            value = value,
            onValueChanged = onValueChanged,
            onDismiss = { showDialog = false }
        )
    }
}
