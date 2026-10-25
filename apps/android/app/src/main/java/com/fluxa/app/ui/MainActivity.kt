package com.fluxa.app.ui

import android.app.Activity
import android.app.PendingIntent
import android.app.PictureInPictureParams
import android.app.RemoteAction
import android.content.BroadcastReceiver
import android.content.Context
import android.content.IntentFilter
import android.graphics.drawable.Icon
import android.util.Rational
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.fluxa.app.ui.rust.NativeVideoHost
import com.google.gson.Gson

class MainActivity : Activity() {

    private lateinit var host: NativeVideoHost
    private val gson = Gson()
    private var pipState = 0

    private val pipToggle = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            host.renderer.pushAction(gson.toJson(mapOf("type" to "mediaCommand", "command" to "toggle", "value" to 0)))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, com.fluxa.app.BuildConfig.IS_TV)
        @Suppress("DEPRECATION")
        window.navigationBarColor = android.graphics.Color.TRANSPARENT
        if (Build.VERSION.SDK_INT >= 29) window.isNavigationBarContrastEnforced = false

        host = NativeVideoHost(this)
        host.onPlayingChanged = ::setPlaying
        host.renderer.onPlayerScreenChanged = { onPlayer ->
            if (!com.fluxa.app.BuildConfig.IS_TV) {
                requestedOrientation = if (onPlayer) {
                    android.content.pm.ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                } else {
                    android.content.pm.ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
                }
            }
        }
        host.renderer.sessionDataDir = filesDir.resolve("fluxa-native").absolutePath
        host.renderer.legacyProfilesJson = { legacyProfilesJson(this) }
        host.renderer.onAppIcon = { AppIcons.apply(this, it) }
        host.renderer.onOpenUrl = { url ->
            runCatching { startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) }
        }
        host.renderer.onExternalPlay = { url ->
            val view = Intent(Intent.ACTION_VIEW).setDataAndType(Uri.parse(url), "video/*")
            runCatching { startActivity(Intent.createChooser(view, null)) }
        }
        host.renderer.onInstallApk = { path -> ApkInstaller.install(this, path) }
        host.renderer.onNativeAction = { Log.i("FluxaNativeRenderer", "Unhandled native actions: $it") }
        host.renderer.dispatchCoreCommand(
            gson.toJson(mapOf("formFactor" to if (com.fluxa.app.BuildConfig.IS_TV) "tv" else "mobile"))
        )
        host.renderer.onPipStateChanged = { state ->
            pipState = state
            updatePipParams()
        }
        if (Build.VERSION.SDK_INT >= 26 && !com.fluxa.app.BuildConfig.IS_TV) {
            val filter = IntentFilter(PIP_TOGGLE)
            ContextCompat.registerReceiver(this, pipToggle, filter, ContextCompat.RECEIVER_NOT_EXPORTED)
        }
        setContentView(host)
        host.renderer.requestFocus()

        if (Build.VERSION.SDK_INT >= 33) {
            val callback = android.window.OnBackInvokedCallback { back() }
            var registered = false
            host.renderer.onBackAvailable = { available ->
                if (available && !registered) {
                    onBackInvokedDispatcher.registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT, callback)
                } else if (!available && registered) {
                    onBackInvokedDispatcher.unregisterOnBackInvokedCallback(callback)
                }
                registered = available
            }
            host.renderer.onBackAvailable?.invoke(true)
        }
        handleIntent(intent)
    }

    override fun onResume() {
        super.onResume()
        ApkInstaller.resume(this)
    }

    override fun onDestroy() {
        if (Build.VERSION.SDK_INT >= 26 && !com.fluxa.app.BuildConfig.IS_TV) unregisterReceiver(pipToggle)
        super.onDestroy()
    }

    private fun pipParams(): PictureInPictureParams? {
        if (Build.VERSION.SDK_INT < 26 || com.fluxa.app.BuildConfig.IS_TV) return null
        val paused = pipState == 2
        val intent = PendingIntent.getBroadcast(
            this, 0, Intent(PIP_TOGGLE).setPackage(packageName),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val icon = Icon.createWithResource(
            this,
            if (paused) android.R.drawable.ic_media_play else android.R.drawable.ic_media_pause,
        )
        val builder = PictureInPictureParams.Builder()
            .setAspectRatio(Rational(16, 9))
            .setActions(listOf(RemoteAction(icon, "Play/Pause", "Play/Pause", intent)))
        if (Build.VERSION.SDK_INT >= 31) {
            builder.setAutoEnterEnabled(pipState != 0)
            builder.setSeamlessResizeEnabled(true)
        }
        return builder.build()
    }

    private fun updatePipParams() {
        val params = pipParams() ?: return
        setPictureInPictureParams(params)
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        if (pipState == 0) return
        val params = pipParams() ?: return
        enterPictureInPictureMode(params)
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: android.content.res.Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        host.renderer.setPictureInPicture(isInPictureInPictureMode)
    }

    override fun onStop() {
        super.onStop()
        if (!com.fluxa.app.BuildConfig.IS_TV) return
        val feed = host.renderer.launcherFeed() ?: return
        val context = applicationContext
        Thread { TvLauncher.publish(context, feed) }.start()
    }

    override fun onKeyUp(keyCode: Int, event: android.view.KeyEvent): Boolean {
        if (keyCode == android.view.KeyEvent.KEYCODE_BACK && Build.VERSION.SDK_INT < 33 && !event.isCanceled) {
            back()
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

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
            host.renderer.pushAction(gson.toJson(mapOf("type" to "search", "query" to query)))
        }
        val data = intent.data
        if (intent.action == Intent.ACTION_VIEW && (data?.scheme == "content" || data?.scheme == "file")) {
            openFile(data)
            return
        }
        if (intent.action == Intent.ACTION_VIEW && data != null) {
            host.renderer.pushAction(gson.toJson(mapOf("type" to "openLink", "url" to data.toString())))
        }
    }

    private fun openFile(uri: Uri) {
        val title = fileName(uri)
        val source = if (uri.scheme == "content") {
            val fd = runCatching { contentResolver.openFileDescriptor(uri, "r")?.detachFd() }.getOrNull() ?: return
            "fd://$fd"
        } else {
            uri.path ?: return
        }
        host.renderer.pushAction(gson.toJson(mapOf("type" to "openFile", "uri" to source, "title" to title)))
    }

    private fun fileName(uri: Uri): String {
        if (uri.scheme == "content") {
            contentResolver.query(uri, arrayOf(android.provider.OpenableColumns.DISPLAY_NAME), null, null, null)?.use {
                if (it.moveToFirst()) it.getString(0)?.let { name -> return name.substringBeforeLast('.') }
            }
        }
        return uri.lastPathSegment?.substringAfterLast('/')?.substringBeforeLast('.') ?: ""
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

private const val PIP_TOGGLE = "com.fluxa.app.PIP_TOGGLE"
