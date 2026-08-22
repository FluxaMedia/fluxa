package com.fluxa.app.shared.feature.addonstore

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun AddonStoreScreen(
    state: AddonStoreUiState,
    language: String?,
    onAction: (AddonStoreAction) -> Unit,
    onConfigureRequested: (String) -> Unit,
    onBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val accentColor = Color(state.accentColorArgb.toInt())

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 20.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                    var backFocused by remember { mutableStateOf(false) }
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape)
                            .onFocusChanged { backFocused = it.isFocused }
                            .background(if (backFocused) Color.White else Color.White.copy(alpha = 0.05f))
                            .clickable(onClick = onBackRequested),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = null,
                            tint = if (backFocused) Color.Black else Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                    Text(
                        text = AppStrings.t(language, "auto.addons"),
                        color = Color.White,
                        fontWeight = FontWeight.Black,
                        fontSize = 26.sp
                    )
                }
            }

            item {
                AddonSmartInput(
                    state = state,
                    language = language,
                    onTextChange = { onAction(AddonStoreAction.InputChanged(it)) },
                    onSubmit = { onAction(AddonStoreAction.SubmitInput) }
                )
            }

            if (state.installedAddons.isNotEmpty()) {
                item {
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = AppStrings.t(language, "auto.installed_stremio_addons"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 16.sp
                    )
                }
                itemsIndexed(
                    items = state.installedAddons,
                    key = { index, addon -> "${addon.url}:$index" }
                ) { _, addon ->
                    InstalledAddonItem(
                        addon = addon,
                        language = language,
                        accentColor = accentColor,
                        onConfigure = addon.configUrl?.let { url -> { onConfigureRequested(url) } },
                        onMoveUp = { onAction(AddonStoreAction.AddonMoved(addon.url, -1)) },
                        onMoveDown = { onAction(AddonStoreAction.AddonMoved(addon.url, 1)) },
                        onRefresh = { onAction(AddonStoreAction.AddonRefreshed(addon.url)) },
                        onToggleEnabled = { enabled -> onAction(AddonStoreAction.AddonToggled(addon.url, enabled)) },
                        onRemove = { onAction(AddonStoreAction.AddonRemoved(addon.url)) }
                    )
                }
            }
        }

        val addedAddonName = state.addedAddonName
        if (addedAddonName != null) {
            AlertDialog(
                onDismissRequest = { onAction(AddonStoreAction.AddedAddonDialogDismissed) },
                title = { Text(AppStrings.t(language, "addons.added_title"), color = Color.White, fontWeight = FontWeight.Bold) },
                text = { Text(AppStrings.format(language, "addons.added_message", addedAddonName), color = Color.White.copy(alpha = 0.8f)) },
                confirmButton = {
                    androidx.compose.material3.TextButton(onClick = { onAction(AddonStoreAction.AddedAddonDialogDismissed) }) {
                        Text(AppStrings.t(language, "common.ok"), color = Color.White)
                    }
                },
                containerColor = FluxaColors.surfaceRaised
            )
        }
    }
}

@Composable
private fun AddonSmartInput(
    state: AddonStoreUiState,
    language: String?,
    onTextChange: (String) -> Unit,
    onSubmit: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        OutlinedTextField(
            value = state.inputText,
            onValueChange = onTextChange,
            modifier = Modifier.fillMaxWidth().height(52.dp),
            singleLine = true,
            placeholder = {
                Text(
                    AppStrings.t(language, "addons.paste_manifest_url"),
                    color = Color.White.copy(alpha = 0.3f),
                    fontSize = 13.sp
                )
            },
            trailingIcon = if (state.isSubmittingInput) {
                {
                    CircularProgressIndicator(
                        color = Color.White.copy(alpha = 0.72f),
                        strokeWidth = 2.dp,
                        modifier = Modifier.size(18.dp)
                    )
                }
            } else null,
            colors = OutlinedTextFieldDefaults.colors(
                unfocusedBorderColor = Color.White.copy(alpha = 0.1f),
                focusedTextColor = Color.White,
                unfocusedTextColor = Color.White,
                cursorColor = Color.White,
                focusedContainerColor = Color.Black.copy(alpha = 0.2f),
                unfocusedContainerColor = Color.Black.copy(alpha = 0.2f)
            ),
            shape = RoundedCornerShape(12.dp),
            isError = state.inputError != null
        )
        val error = state.inputError
        if (error != null) {
            Text(text = error, color = FluxaColors.errorRed, fontSize = 12.sp)
        }
        if (!state.isSubmittingInput &&
            state.inputDetectedType != AddonStoreInputType.UNKNOWN &&
            state.inputDetectedType != AddonStoreInputType.SEARCH_QUERY
        ) {
            androidx.compose.material3.Button(
                onClick = onSubmit,
                enabled = state.inputText.isNotBlank(),
                modifier = Modifier.fillMaxWidth().height(44.dp),
                shape = RoundedCornerShape(12.dp),
                colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
            ) {
                Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(6.dp))
                Text(AppStrings.t(language, "addons.add"), fontWeight = FontWeight.Bold, fontSize = 14.sp)
            }
        }
    }
}

@Composable
private fun InstalledAddonItem(
    addon: InstalledAddonUiModel,
    language: String?,
    accentColor: Color,
    onConfigure: (() -> Unit)?,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onRefresh: () -> Unit,
    onToggleEnabled: (Boolean) -> Unit,
    onRemove: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(
                if (addon.isEnabled) Color.White.copy(alpha = 0.04f) else Color.White.copy(alpha = 0.025f)
            )
            .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(18.dp))
            .padding(horizontal = 18.dp, vertical = 16.dp)
    ) {
        Row(modifier = Modifier.fillMaxWidth().padding(end = 56.dp), verticalAlignment = Alignment.Top) {
            AddonLogo(url = addon.logoUrl, size = 70.dp, contentDescription = addon.name)
            Spacer(Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = addon.name,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                if (addon.canRemove) {
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                        if (onConfigure != null) {
                            AddonIconButton(icon = Icons.Filled.Settings, onClick = onConfigure)
                        }
                        if (addon.isRefreshing) {
                            Box(modifier = Modifier.size(36.dp), contentAlignment = Alignment.Center) {
                                CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(18.dp))
                            }
                        } else {
                            AddonIconButton(icon = Icons.Filled.Refresh, onClick = onRefresh)
                        }
                        AddonIconButton(
                            icon = Icons.Filled.KeyboardArrowUp,
                            enabled = addon.canMoveUp,
                            tint = Color.White.copy(alpha = if (addon.canMoveUp) 0.68f else 0.22f),
                            onClick = onMoveUp
                        )
                        AddonIconButton(
                            icon = Icons.Filled.KeyboardArrowDown,
                            enabled = addon.canMoveDown,
                            tint = Color.White.copy(alpha = if (addon.canMoveDown) 0.68f else 0.22f),
                            onClick = onMoveDown
                        )
                        AddonIconButton(icon = Icons.Filled.Close, tint = Color.White.copy(alpha = 0.62f), onClick = onRemove)
                    }
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    text = addon.description,
                    color = Color.White.copy(alpha = 0.48f),
                    fontSize = 12.sp,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
        if (addon.canRemove) {
            Switch(
                checked = addon.isEnabled,
                onCheckedChange = onToggleEnabled,
                modifier = Modifier.align(Alignment.TopEnd).height(40.dp),
                colors = SwitchDefaults.colors(
                    checkedTrackColor = accentColor,
                    uncheckedThumbColor = Color.White.copy(alpha = 0.72f),
                    uncheckedTrackColor = Color.White.copy(alpha = 0.18f)
                )
            )
        }
    }
}

@Composable
private fun AddonIconButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    enabled: Boolean = true,
    tint: Color = Color.White.copy(alpha = 0.68f),
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .size(36.dp)
            .clip(CircleShape)
            .onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White else Color.Transparent)
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, contentDescription = null, tint = if (focused) Color.Black else tint, modifier = Modifier.size(18.dp))
    }
}

@Composable
private fun AddonLogo(url: String?, size: androidx.compose.ui.unit.Dp, contentDescription: String?) {
    Box(
        modifier = Modifier.size(size).clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.08f)),
        contentAlignment = Alignment.Center
    ) {
        if (!url.isNullOrBlank()) {
            FluxaRemoteImage(
                imageUrl = url,
                cacheKey = "addon-logo:$url",
                contentDescription = contentDescription,
                modifier = Modifier.size(size * 0.82f).clip(RoundedCornerShape(8.dp)),
                contentScale = androidx.compose.ui.layout.ContentScale.Fit
            )
        }
    }
}
