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

data class CommunityAddon(
    val name: String,
    val description: String,
    val url: String,
    val logoUrl: String? = null,
    val version: String? = null,
    val manifest: AddonManifest? = null,
    val configUrl: String? = null,
    val configurable: Boolean = false,
    val fallbackIcon: ImageVector = FluxaIcons.Extension,
    val type: AddonType = AddonType.STREMIO
)

enum class AddonType {
    STREMIO,
    CLOUDSTREAM3_REPO,
    CLOUDSTREAM3_PLUGIN
}

internal data class AddonSearchState(
    val results: List<CommunityAddon> = emptyList(),
    val loading: Boolean = false,
    val errorKey: String? = null
)

internal data class SmartInputState(
    val text: String = "",
    val detectedType: DetectedType = DetectedType.UNKNOWN,
    val isLoading: Boolean = false,
    val error: String? = null
)

internal enum class DetectedType {
    UNKNOWN,
    STREMIO_MANIFEST,
    CLOUDSTREAM_REPO,
    SEARCH_QUERY
}

internal val addonStoreClient by lazy {
    OkHttpClient.Builder()
        .addInterceptor { chain ->
            chain.proceed(HttpRequestSecurity.upgradeRemoteHttpRequest(chain.request()))
        }
        .callTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
        .build()
}

internal data class AddonSearchCacheEntry(
    val createdAt: Long,
    val results: List<CommunityAddon>
)

internal const val ADDON_SEARCH_CACHE_MS = 10 * 60 * 1000L
internal val addonSearchCache = ConcurrentHashMap<String, AddonSearchCacheEntry>()

