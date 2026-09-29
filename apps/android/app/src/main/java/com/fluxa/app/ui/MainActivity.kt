package com.fluxa.app.ui

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.fluxa.app.ui.rust.NativeVideoHost
import com.google.gson.Gson

class MainActivity : Activity() {

    private lateinit var host: NativeVideoHost
    private val gson = Gson()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, com.fluxa.app.BuildConfig.IS_TV)
        @Suppress("DEPRECATION")
        window.navigationBarColor = android.graphics.Color.TRANSPARENT
        if (Build.VERSION.SDK_INT >= 29) window.isNavigationBarContrastEnforced = false

        host = NativeVideoHost(this)
        host.onPlayingChanged = ::setPlaying
        host.renderer.sessionDataDir = filesDir.resolve("fluxa-native").absolutePath
        host.renderer.legacyProfilesJson = { legacyProfilesJson(this) }
        host.renderer.onAppIcon = { AppIcons.apply(this, it) }
        host.renderer.onOpenUrl = { url ->
            runCatching { startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) }
        }
        host.renderer.onNativeAction = { Log.i("FluxaNativeRenderer", "Unhandled native actions: $it") }
        host.renderer.dispatchCoreCommand(
            gson.toJson(mapOf("formFactor" to if (com.fluxa.app.BuildConfig.IS_TV) "tv" else "mobile"))
        )
        setContentView(host)
        host.renderer.requestFocus()

        if (Build.VERSION.SDK_INT >= 33) {
            onBackInvokedDispatcher.registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT, ::back)
        }
        handleIntent(intent)
    }

    @Deprecated("Replaced by OnBackInvokedCallback on API 33+")
    override fun onBackPressed() = back()

    private fun back() {
        if (!host.renderer.back()) moveTaskToBack(true)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        if (intent == null) return
        val query = intent.getStringExtra(android.app.SearchManager.QUERY) ?: intent.getStringExtra("query")
        if (query != null && intent.action in searchActions) {
            host.renderer.pushAction(
                gson.toJson(
                    mapOf(
                        "type" to "discoverCatalog",
                        "contentType" to "movie",
                        "catalogKey" to "",
                        "extraName" to "",
                        "extraValue" to "",
                        "query" to query,
                    )
                )
            )
        }
        val data = intent.data
        if (intent.action == Intent.ACTION_VIEW && data?.scheme == "fluxa" && data.host == "oauth") {
            host.renderer.pushAction(gson.toJson(mapOf("type" to "oauthCallback", "url" to data.toString())))
            return
        }
        if (intent.action == Intent.ACTION_VIEW && data?.scheme == "app" && data.host == "play") {
            val id = data.getQueryParameter("id")?.takeIf(String::isNotBlank) ?: return
            val type = data.getQueryParameter("type")?.takeIf(String::isNotBlank) ?: return
            host.renderer.pushAction(gson.toJson(mapOf("type" to "detail", "id" to id, "itemType" to type)))
        }
    }

    private fun setPlaying(playing: Boolean) {
        if (playing) window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        else window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        if (com.fluxa.app.BuildConfig.IS_TV) return
        val controller = WindowInsetsControllerCompat(window, window.decorView)
        if (playing) {
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
        } else {
            controller.show(WindowInsetsCompat.Type.systemBars())
        }
    }
}

private val searchActions = setOf(
    Intent.ACTION_SEARCH,
    "com.google.android.gms.actions.SEARCH_ACTION",
    "com.google.android.gms.actions.SEARCH_AND_PLAY",
    "android.media.action.MEDIA_PLAY_FROM_SEARCH",
)
