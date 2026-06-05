@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.util.Log
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.RepositoryManifest
import com.fluxa.app.data.repository.HttpRequestSecurity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.jsoup.Jsoup
import java.io.IOException
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

internal fun androidx.compose.foundation.lazy.LazyListScope.addAddonSmartInputSection(
    lang: String,
    smartInput: SmartInputState,
    onTextChange: (String) -> Unit,
    onSubmit: () -> Unit
) {
    item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(18.dp))
                        .background(Color.White.copy(alpha = 0.04f))
                        .padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Box(
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        OutlinedTextField(
                            value = smartInput.text,
                            onValueChange = { 
                                onTextChange(it) 
                            },
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(52.dp),
                            singleLine = true,
                            leadingIcon = {
                                Icon(
                                    when (smartInput.detectedType) {
                                        DetectedType.CLOUDSTREAM_REPO -> FluxaIcons.Cloud
                                        else -> FluxaIcons.Link
                                    },
                                    null,
                                    tint = when (smartInput.detectedType) {
                                        DetectedType.STREMIO_MANIFEST -> Color(0xFF4CAF50)
                                        DetectedType.CLOUDSTREAM_REPO -> Color(0xFF2196F3)
                                        else -> Color.White.copy(alpha = 0.42f)
                                    }
                                )
                            },
                            placeholder = {
                                Text(
                                    AppStrings.t(lang, "addons.paste_manifest_url"),
                                    color = Color.White.copy(alpha = 0.3f),
                                    fontSize = 13.sp
                                )
                            },
                            trailingIcon = {
                                if (smartInput.isLoading) {
                                    CircularProgressIndicator(
                                        color = Color.White.copy(alpha = 0.72f),
                                        strokeWidth = 2.dp,
                                        modifier = Modifier.size(18.dp)
                                    )
                                } else if (smartInput.detectedType == DetectedType.STREMIO_MANIFEST || smartInput.detectedType == DetectedType.CLOUDSTREAM_REPO) {
                                    IconButton(
                                        enabled = smartInput.text.isNotBlank(),
                                        onClick = onSubmit
                                    ) {
                                        Icon(
                                            FluxaIcons.Add,
                                            null,
                                            tint = if (smartInput.detectedType == DetectedType.STREMIO_MANIFEST) Color(0xFF4CAF50) else Color(0xFF2196F3),
                                            modifier = Modifier.size(20.dp)
                                        )
                                    }
                                }
                            },
                            colors = OutlinedTextFieldDefaults.colors(
                                focusedBorderColor = when (smartInput.detectedType) {
                                    DetectedType.STREMIO_MANIFEST -> Color(0xFF4CAF50).copy(alpha = 0.5f)
                                    DetectedType.CLOUDSTREAM_REPO -> Color(0xFF2196F3).copy(alpha = 0.5f)
                                    else -> Color.White.copy(alpha = 0.2f)
                                },
                                unfocusedBorderColor = Color.White.copy(alpha = 0.1f),
                                focusedTextColor = Color.White,
                                unfocusedTextColor = Color.White,
                                cursorColor = Color.White,
                                focusedContainerColor = Color.Black.copy(alpha = 0.2f),
                                unfocusedContainerColor = Color.Black.copy(alpha = 0.2f)
                            ),
                            shape = RoundedCornerShape(12.dp),
                            isError = smartInput.error != null
                        )
                    }
                    
                    // Error message
                    val error = smartInput.error
                    if (error != null) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            Icon(
                                FluxaIcons.Error,
                                null,
                                tint = Color(0xFFEF5350),
                                modifier = Modifier.size(16.dp)
                            )
                            Text(
                                text = error,
                                color = Color(0xFFEF5350),
                                fontSize = 12.sp
                            )
                        }
                    }
                }
            }}
