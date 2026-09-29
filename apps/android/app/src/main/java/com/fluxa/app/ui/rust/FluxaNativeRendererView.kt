package com.fluxa.app.ui.rust

import android.content.Context
import android.graphics.Rect
import android.os.Handler
import android.os.HandlerThread
import android.view.KeyEvent
import android.view.InputDevice
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeProvider
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.google.gson.Gson
import java.io.File
import java.io.FileOutputStream
import java.security.MessageDigest
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONArray
import org.json.JSONObject

class FluxaNativeRendererView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    @Volatile private var nativeHandle: Long = 0L
    private var pendingHomeStateJson: String? = null
    private var pendingCoreSnapshotJson: String? = null
    private var pendingFormFactor: String? = null
    private var lastCoreCommandJson: String? = null
    private var safeBottomInsetPx = 0
    private var accessibilityHeroTitle = "Recommended"
    private var accessibilityCardTitles = emptyList<String>()
    var sessionDataDir: String? = null
    var legacyProfilesJson: (() -> String)? = null
    private val pendingActions = mutableListOf<String>()
    var onNativeAction: ((String) -> Unit)? = null
    var onCoreCommand: ((String) -> Unit)? = null
    var onVideoRequest: ((String) -> Unit)? = null
    var onAppIcon: ((String) -> Unit)? = null
    var onOpenUrl: ((String) -> Unit)? = null
    private var renderThread: HandlerThread? = null
    private var renderHandler: Handler? = null
    private val artworkScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val artworkSemaphore = Semaphore(4)
    private val artworkClient = OkHttpClient.Builder()
        .callTimeout(20, TimeUnit.SECONDS)
        .build()
    private val artworkRequests = ConcurrentHashMap.newKeySet<String>()
    private val resolvedArtworkFiles = ConcurrentHashMap<String, String>()
    private val artworkCacheDirectory = context.cacheDir.resolve("native-artwork-host")
    private val gson = Gson()
    private val renderFrame = object : Runnable {
        override fun run() {
            if (nativeHandle == 0L) return
            val frameStartedAt = android.os.SystemClock.uptimeMillis()
            NativeRenderer.renderNative(nativeHandle)
            val actions = NativeRenderer.pollActionsNative(nativeHandle)
            if (actions.isNotBlank() && actions != "[]") {
                post { onNativeAction?.invoke(actions) }
            }
            val appIcon = NativeRenderer.takeAppIconNative(nativeHandle)
            if (!appIcon.isNullOrEmpty()) {
                post { onAppIcon?.invoke(appIcon) }
            }
            val openUrl = NativeRenderer.takeOpenUrlNative(nativeHandle)
            if (!openUrl.isNullOrEmpty()) {
                post { onOpenUrl?.invoke(openUrl) }
            }
            val videoRequests = NativeRenderer.pollVideoNative()
            if (videoRequests != "[]") {
                post { onVideoRequest?.invoke(videoRequests) }
            }
            // Keep a 60 Hz cadence measured from frame start. Waiting a full
            // 16 ms after rendering compounded render cost (about 8 ms on a
            // mid-range phone) into ~24 ms frame intervals / ~42 FPS.
            val elapsed = android.os.SystemClock.uptimeMillis() - frameStartedAt
            renderHandler?.postDelayed(this, (16L - elapsed).coerceAtLeast(0L))
        }
    }

    init {
        holder.addCallback(this)
        holder.setFormat(android.graphics.PixelFormat.TRANSLUCENT)
        setZOrderMediaOverlay(true)
        ViewCompat.setOnApplyWindowInsetsListener(this) { _, insets ->
            safeBottomInsetPx = insets
                .getInsets(WindowInsetsCompat.Type.navigationBars())
                .bottom
            syncSafeInsets()
            val imeVisible = insets.isVisible(WindowInsetsCompat.Type.ime())
            if (imeShown && !imeVisible && nativeHandle != 0L) {
                NativeRenderer.blurTextInputNative(nativeHandle)
            }
            imeShown = imeVisible
            insets
        }
        isFocusable = true
        isFocusableInTouchMode = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
    }

    /** Pushes the platform data-source projection into the Rust-owned scene. */
    fun setHomeStateJson(json: String) {
        pendingHomeStateJson = json
        updateAccessibilityState(json)
        scheduleArtworkDownloads(json)
        publishResolvedHomeState()
    }

    /** Dispatches intent to the Android app-scoped Core runtime. */
    fun dispatchCoreCommand(json: String) {
        if (json == lastCoreCommandJson) return
        lastCoreCommandJson = json
        val formFactor = runCatching { JSONObject(json).optString("formFactor") }
            .getOrNull()
            ?.takeIf { it in setOf("mobile", "tv", "desktop") }
        if (formFactor != null) {
            pendingFormFactor = formFactor
            val handle = nativeHandle
            if (handle != 0L) NativeRenderer.setFormFactorNative(handle, formFactor)
        }
        onCoreCommand?.invoke(json)
    }

    fun pushAction(json: String) {
        val handle = nativeHandle
        if (handle == 0L) pendingActions += json else NativeRenderer.pushActionNative(handle, json)
    }

    fun setCoreSnapshotJson(json: String) {
        pendingCoreSnapshotJson = json
        publishCoreSnapshot()
    }

    fun reportVideoStatus(
        position: Double,
        duration: Double,
        paused: Boolean,
        muted: Boolean,
        hasFrame: Boolean,
        buffering: Float,
        error: String?,
    ) {
        NativeRenderer.videoStatusNative(position, duration, paused, muted, hasFrame, buffering, error)
    }

    fun coreSnapshotJson(): String =
        if (nativeHandle == 0L) "{}" else NativeRenderer.snapshotNative(nativeHandle)

    override fun getAccessibilityNodeProvider(): AccessibilityNodeProvider = accessibilityProvider

    override fun surfaceCreated(holder: SurfaceHolder) {
        if (nativeHandle == 0L) {
            nativeHandle = NativeRenderer.createNative(
                resources.displayMetrics.density,
                context.cacheDir.resolve("native-artwork").absolutePath,
            )
            sessionDataDir?.let { dir ->
                legacyProfilesJson?.invoke()?.let { NativeRenderer.importLegacyNative(dir, it) }
                NativeRenderer.startSessionNative(nativeHandle, dir)
            }
            pendingActions.forEach { NativeRenderer.pushActionNative(nativeHandle, it) }
            pendingActions.clear()
        }
        requestApplyInsets()
        syncSafeInsets()
        pendingHomeStateJson?.let { publishResolvedHomeState() }
        pendingCoreSnapshotJson?.let { publishCoreSnapshot() }
        pendingFormFactor?.let { NativeRenderer.setFormFactorNative(nativeHandle, it) }
        NativeRenderer.surfaceCreatedNative(nativeHandle, holder.surface, width, height)
        startRenderThread()
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        if (nativeHandle != 0L) {
            NativeRenderer.surfaceChangedNative(nativeHandle, width, height)
            syncSafeInsets()
        }
    }

    private fun syncSafeInsets() {
        val handle = nativeHandle
        if (handle == 0L) return
        val density = resources.displayMetrics.density.coerceAtLeast(1f)
        NativeRenderer.setSafeBottomInsetNative(handle, safeBottomInsetPx / density)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        stopRenderThread()
        if (nativeHandle != 0L) NativeRenderer.surfaceDestroyedNative(nativeHandle)
    }

    private fun startRenderThread() {
        stopRenderThread()
        val thread = HandlerThread("fluxa-native-renderer").also { it.start() }
        renderThread = thread
        renderHandler = Handler(thread.looper).also { it.post(renderFrame) }
    }

    private fun stopRenderThread() {
        renderHandler?.removeCallbacksAndMessages(null)
        renderHandler = null
        renderThread?.quitSafely()
        renderThread = null
    }

    override fun onDetachedFromWindow() {
        stopRenderThread()
        artworkScope.cancel()
        if (nativeHandle != 0L) {
            NativeRenderer.destroyNative(nativeHandle)
            nativeHandle = 0L
        }
        super.onDetachedFromWindow()
    }

    private fun scheduleArtworkDownloads(json: String) {
        val urls = linkedSetOf<String>()
        runCatching { collectArtworkUrls(JSONObject(json), urls) }
        urls.take(MAX_ARTWORK_URLS).forEach { sourceUrl ->
            val existing = artworkFile(sourceUrl)
            if (existing.isFile) {
                resolvedArtworkFiles[sourceUrl] = existing.absolutePath
                return@forEach
            }
            if (!artworkRequests.add(sourceUrl)) return@forEach
            artworkScope.launch {
                artworkSemaphore.withPermit {
                    runCatching { downloadArtwork(sourceUrl, existing) }
                        .onSuccess {
                            resolvedArtworkFiles[sourceUrl] = existing.absolutePath
                            post { publishResolvedHomeState() }
                        }
                        .onFailure { cause ->
                            artworkRequests.remove(sourceUrl)
                            File(artworkCacheDirectory, "${existing.name}.part").delete()
                            android.util.Log.w(
                                "FluxaNativeRenderer",
                                "Artwork fetch failed (${cause.javaClass.simpleName})",
                            )
                        }
                }
            }
        }
    }

    private fun collectArtworkUrls(value: Any?, output: MutableSet<String>) {
        when (value) {
            is JSONObject -> {
                val keys = value.keys()
                while (keys.hasNext()) {
                    val key = keys.next()
                    val field = value.opt(key)
                    val url = if (key in ARTWORK_URL_FIELDS && field is String) {
                        normalizeArtworkUrl(field)
                    } else {
                        null
                    }
                    if (url != null) output.add(url) else collectArtworkUrls(field, output)
                }
            }
            is JSONArray -> for (index in 0 until value.length()) {
                collectArtworkUrls(value.opt(index), output)
            }
        }
    }

    private fun normalizeArtworkUrl(value: String): String? {
        val url = value.trim().let { if (it.startsWith("//")) "https:$it" else it }
        return url.takeIf { it.startsWith("https://") || it.startsWith("http://") }
    }

    private fun artworkFile(url: String): File {
        val digest = MessageDigest.getInstance("SHA-256")
            .digest(url.toByteArray())
            .joinToString("") { byte -> "%02x".format(byte) }
        return File(artworkCacheDirectory, "$digest.artwork")
    }

    private fun downloadArtwork(url: String, destination: File) {
        artworkCacheDirectory.mkdirs()
        val temporary = File(artworkCacheDirectory, "${destination.name}.part")
        val request = Request.Builder().url(url).header("Accept-Encoding", "identity").build()
        artworkClient.newCall(request).execute().use { response ->
            check(response.isSuccessful) { "HTTP ${response.code}" }
            val body = requireNotNull(response.body) { "Empty response" }
            check(body.contentLength() <= MAX_ARTWORK_BYTES || body.contentLength() == -1L) {
                "Artwork response exceeds size limit"
            }
            body.byteStream().use { input ->
                FileOutputStream(temporary).use { output ->
                    val buffer = ByteArray(16 * 1024)
                    var total = 0L
                    while (true) {
                        val count = input.read(buffer)
                        if (count < 0) break
                        total += count
                        check(total <= MAX_ARTWORK_BYTES) { "Artwork response exceeds size limit" }
                        output.write(buffer, 0, count)
                    }
                }
            }
        }
        if (!temporary.renameTo(destination)) {
            temporary.delete()
            check(destination.isFile) { "Could not publish artwork cache file" }
        }
    }

    private fun publishResolvedHomeState() {
        val handle = nativeHandle
        val source = pendingHomeStateJson
        if (handle == 0L || source == null) return
        val resolved = runCatching {
            resolveArtworkUrls(JSONObject(source)).toString()
        }.getOrDefault(source)
        NativeRenderer.setHomeStateNative(handle, resolved)
    }

    private fun publishCoreSnapshot() {
        val handle = nativeHandle
        val source = pendingCoreSnapshotJson
        if (handle == 0L || source == null) return
        val resolved = runCatching {
            resolveArtworkUrls(JSONObject(source)).toString()
        }.getOrDefault(source)
        NativeRenderer.setCoreSnapshotNative(handle, resolved)
    }

    private fun resolveArtworkUrls(value: Any?): Any? = when (value) {
        is JSONObject -> JSONObject().apply {
            val keys = value.keys()
            while (keys.hasNext()) {
                val key = keys.next()
                val field = value.opt(key)
                val sourceUrl = if (key in ARTWORK_URL_FIELDS && field is String) {
                    normalizeArtworkUrl(field)
                } else {
                    null
                }
                val filePath = sourceUrl?.let(resolvedArtworkFiles::get)
                put(key, if (filePath != null) "file://$filePath" else resolveArtworkUrls(field))
            }
        }
        is JSONArray -> JSONArray().apply {
            for (index in 0 until value.length()) put(resolveArtworkUrls(value.opt(index)))
        }
        else -> value
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (nativeHandle == 0L) return false
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                requestFocus()
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_DOWN, logicalX(event.x), logicalY(event.y))
            }
            MotionEvent.ACTION_MOVE ->
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_MOVE, logicalX(event.x), logicalY(event.y))
            MotionEvent.ACTION_UP -> {
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_UP, logicalX(event.x), logicalY(event.y))
                performClick()
                syncSoftKeyboard()
            }
            MotionEvent.ACTION_CANCEL -> {
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_UP, logicalX(event.x), logicalY(event.y))
                return true
            }
            else -> return false
        }
        return true
    }

    override fun onGenericMotionEvent(event: MotionEvent): Boolean {
        val handle = nativeHandle
        if (handle != 0L && event.isFromSource(InputDevice.SOURCE_CLASS_POINTER)) {
            when (event.actionMasked) {
                MotionEvent.ACTION_HOVER_MOVE -> {
                    NativeRenderer.pointerEventNative(handle, POINTER_MOVE, logicalX(event.x), logicalY(event.y))
                    return true
                }
                MotionEvent.ACTION_SCROLL -> {
                    NativeRenderer.scrollNative(handle, -event.getAxisValue(MotionEvent.AXIS_VSCROLL) * 48f)
                    return true
                }
            }
        }
        return super.onGenericMotionEvent(event)
    }

    private fun logicalX(value: Float): Float = value / resources.displayMetrics.density

    private fun logicalY(value: Float): Float = value / resources.displayMetrics.density

    override fun performClick(): Boolean {
        super.performClick()
        return true
    }

    private fun updateAccessibilityState(json: String) {
        runCatching {
            val root = JSONObject(json)
            accessibilityHeroTitle = root.optString("title", "Recommended")
            val cards = root.optJSONArray("cards")
            accessibilityCardTitles = if (cards == null) {
                emptyList()
            } else {
                buildList(cards.length()) {
                    for (index in 0 until cards.length()) {
                        add(cards.optJSONObject(index)?.optString("title", "Item") ?: "Item")
                    }
                }
            }
        }
        sendAccessibilityEvent(AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED)
    }

    private val accessibilityProvider = object : AccessibilityNodeProvider() {
        override fun createAccessibilityNodeInfo(virtualViewId: Int): AccessibilityNodeInfo? {
            if (virtualViewId == HOST_VIEW_ID) {
                return AccessibilityNodeInfo.obtain(this@FluxaNativeRendererView).apply {
                    packageName = context.packageName
                    className = FluxaNativeRendererView::class.java.name
                    isVisibleToUser = visibility == VISIBLE
                    for (nodeId in virtualNodeIds()) addChild(this@FluxaNativeRendererView, nodeId)
                }
            }
            if (!virtualNodeIds().contains(virtualViewId)) return null
            val info = AccessibilityNodeInfo.obtain()
            info.packageName = context.packageName
            info.className = "android.widget.Button"
            info.setSource(this@FluxaNativeRendererView, virtualViewId)
            info.setParent(this@FluxaNativeRendererView, HOST_VIEW_ID)
            info.isEnabled = true
            info.isFocusable = true
            info.isVisibleToUser = visibility == VISIBLE
            info.text = accessibilityLabel(virtualViewId)
            info.contentDescription = info.text
            info.setBoundsInParent(accessibilityBounds(virtualViewId))
            info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
            info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_FOCUS)
            info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_ACCESSIBILITY_FOCUS)
            return info
        }

        override fun performAction(virtualViewId: Int, action: Int, arguments: android.os.Bundle?): Boolean {
            if (!virtualNodeIds().contains(virtualViewId) || nativeHandle == 0L) return false
            return when (action) {
                AccessibilityNodeInfo.ACTION_CLICK -> {
                    NativeRenderer.activateNodeNative(nativeHandle, virtualViewId.toLong())
                    true
                }
                AccessibilityNodeInfo.ACTION_FOCUS,
                AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS -> {
                    NativeRenderer.focusNodeNative(nativeHandle, virtualViewId.toLong())
                    true
                }
                AccessibilityNodeInfo.ACTION_CLEAR_FOCUS,
                AccessibilityNodeInfo.ACTION_CLEAR_ACCESSIBILITY_FOCUS -> true
                else -> false
            }
        }
    }

    private fun virtualNodeIds(): List<Int> = buildList {
        addAll(listOf(10, 11, 12, 13, 14, 20, 21))
        addAll(accessibilityCardTitles.indices.map { 100 + it })
    }

    private fun accessibilityLabel(virtualViewId: Int): String = when (virtualViewId) {
        10 -> "Home"
        11 -> "Library"
        12 -> "Discover"
        13 -> "Calendar"
        14 -> "Profile"
        20 -> "Play $accessibilityHeroTitle"
        21 -> "More info about $accessibilityHeroTitle"
        else -> accessibilityCardTitles.getOrNull(virtualViewId - 100) ?: "Item"
    }

    private fun accessibilityBounds(virtualViewId: Int): Rect {
        val topBarWidth = minOf(620f, width.toFloat() - 32f).coerceAtLeast(320f)
        val topX = (width - topBarWidth) / 2f + 10f
        when (virtualViewId) {
            10, 11, 12, 13, 14 -> {
                val index = virtualViewId - 10
                return Rect((topX + index * 122f).toInt(), 24, (topX + index * 122f + 118f).toInt(), 62)
            }
            20, 21 -> {
                val x = if (virtualViewId == 20) 32f else 158f
                val buttonWidth = if (virtualViewId == 20) 118f else 126f
                val y = (height * 0.20f).coerceAtLeast(100f) + 180f
                return Rect(x.toInt(), y.toInt(), (x + buttonWidth).toInt(), (y + 42f).toInt())
            }
        }
        val index = virtualViewId - 100
        val cardWidth = ((width - 64f) / 5.8f).coerceIn(170f, 288f)
        val cardHeight = (cardWidth * 0.56f).coerceAtLeast(104f)
        val x = 16f + index * (cardWidth + 8f)
        val y = height * 0.64f + 33f
        return Rect(x.toInt(), y.toInt(), (x + cardWidth).toInt(), (y + cardHeight).toInt())
    }

    fun back(): Boolean = nativeHandle != 0L && NativeRenderer.backNative(nativeHandle)

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        if (nativeHandle != 0L && isSelectKey(keyCode)) {
            if (event.repeatCount == 0) {
                selectLongPressed = false
                event.startTracking()
            }
            return true
        }
        if (nativeHandle != 0L && keyCode != KeyEvent.KEYCODE_BACK && NativeRenderer.isNavigationKey(keyCode)) {
            NativeRenderer.keyDownNative(nativeHandle, keyCode, if (event.isShiftPressed) 1 else 0)
            return true
        }
        if (nativeHandle != 0L && NativeRenderer.textInputFocusedNative(nativeHandle)) {
            val codePoint = event.unicodeChar
            if (codePoint > 0 && !event.isCtrlPressed && !event.isAltPressed) {
                NativeRenderer.textInputNative(nativeHandle, String(Character.toChars(codePoint)))
                return true
            }
        }
        return super.onKeyDown(keyCode, event)
    }

    private var selectLongPressed = false
    private var imeShown = false

    private fun syncSoftKeyboard() {
        val input = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? android.view.inputmethod.InputMethodManager ?: return
        post {
            if (nativeHandle == 0L) return@post
            if (NativeRenderer.textInputFocusedNative(nativeHandle)) {
                requestFocus()
                input.restartInput(this)
                input.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
            } else {
                input.hideSoftInputFromWindow(windowToken, 0)
            }
        }
    }

    private fun isSelectKey(keyCode: Int) =
        keyCode == KeyEvent.KEYCODE_DPAD_CENTER || keyCode == KeyEvent.KEYCODE_ENTER

    override fun onKeyLongPress(keyCode: Int, event: KeyEvent): Boolean {
        if (nativeHandle != 0L && isSelectKey(keyCode)) {
            selectLongPressed = true
            NativeRenderer.keyDownNative(nativeHandle, KeyEvent.KEYCODE_MENU, 0)
            return true
        }
        return super.onKeyLongPress(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (nativeHandle != 0L && isSelectKey(keyCode)) {
            if (!selectLongPressed && !event.isCanceled) {
                NativeRenderer.keyDownNative(nativeHandle, keyCode, 0)
                syncSoftKeyboard()
            }
            selectLongPressed = false
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

    override fun onCheckIsTextEditor(): Boolean = true

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        outAttrs.inputType = android.text.InputType.TYPE_CLASS_TEXT
        outAttrs.imeOptions = EditorInfo.IME_ACTION_DONE
        val connection = object : BaseInputConnection(this, true) {
            private fun sync(result: Boolean): Boolean {
                if (nativeHandle != 0L) {
                    NativeRenderer.setFocusedTextNative(nativeHandle, editable.toString())
                }
                return result
            }

            override fun commitText(text: CharSequence, newCursorPosition: Int) =
                sync(super.commitText(text, newCursorPosition))

            override fun setComposingText(text: CharSequence, newCursorPosition: Int) =
                sync(super.setComposingText(text, newCursorPosition))

            override fun finishComposingText() = sync(super.finishComposingText())

            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int) =
                sync(super.deleteSurroundingText(beforeLength, afterLength))

            private var batch = 0

            override fun beginBatchEdit(): Boolean {
                batch++
                return super.beginBatchEdit()
            }

            override fun endBatchEdit(): Boolean {
                batch = (batch - 1).coerceAtLeast(0)
                return sync(super.endBatchEdit())
            }

            override fun setComposingRegion(start: Int, end: Int) =
                sync(super.setComposingRegion(start, end))

            override fun setSelection(start: Int, end: Int) =
                sync(super.setSelection(start, end))

            override fun commitCompletion(text: android.view.inputmethod.CompletionInfo) =
                sync(super.commitCompletion(text))

            override fun commitCorrection(correctionInfo: android.view.inputmethod.CorrectionInfo) =
                sync(super.commitCorrection(correctionInfo))

            override fun replaceText(
                start: Int,
                end: Int,
                text: CharSequence,
                newCursorPosition: Int,
                textAttribute: android.view.inputmethod.TextAttribute?,
            ) = sync(super.replaceText(start, end, text, newCursorPosition, textAttribute))

            override fun sendKeyEvent(event: KeyEvent): Boolean {
                if (event.keyCode == KeyEvent.KEYCODE_DEL) {
                    if (event.action == KeyEvent.ACTION_DOWN) {
                        val text = editable ?: return true
                        val end = android.text.Selection.getSelectionEnd(text).takeIf { it > 0 } ?: text.length
                        val start = android.text.Selection.getSelectionStart(text).coerceIn(0, end)
                        if (start != end) text.delete(start, end) else if (end > 0) text.delete(end - 1, end)
                        sync(true)
                    }
                    return true
                }
                return super.sendKeyEvent(event)
            }
        }
        if (nativeHandle != 0L) {
            val current = NativeRenderer.focusedTextNative(nativeHandle)
            connection.editable?.let {
                it.append(current)
                android.text.Selection.setSelection(it, it.length)
            }
            outAttrs.initialSelStart = current.length
            outAttrs.initialSelEnd = current.length
        }
        return connection
    }
}

private object NativeRenderer {
    init {
        System.loadLibrary("fluxa_android_renderer")
    }

    @JvmStatic external fun createNative(density: Float, artworkCacheDir: String): Long
    @JvmStatic external fun setSafeBottomInsetNative(handle: Long, insetDp: Float)
    @JvmStatic external fun setHomeStateNative(handle: Long, json: String)
    @JvmStatic external fun setFormFactorNative(handle: Long, formFactor: String)
    @JvmStatic external fun setCoreSnapshotNative(handle: Long, snapshot: String)
    @JvmStatic external fun snapshotNative(handle: Long): String
    @JvmStatic external fun startSessionNative(handle: Long, dataDir: String): Boolean
    @JvmStatic external fun importLegacyNative(dataDir: String, legacy: String): Boolean
    @JvmStatic external fun pushActionNative(handle: Long, action: String)
    @JvmStatic external fun destroyNative(handle: Long)
    @JvmStatic external fun surfaceCreatedNative(handle: Long, surface: android.view.Surface, width: Int, height: Int)
    @JvmStatic external fun surfaceChangedNative(handle: Long, width: Int, height: Int)
    @JvmStatic external fun surfaceDestroyedNative(handle: Long)
    @JvmStatic external fun renderNative(handle: Long)
    @JvmStatic external fun pollActionsNative(handle: Long): String
    @JvmStatic external fun takeAppIconNative(handle: Long): String?
    @JvmStatic external fun takeOpenUrlNative(handle: Long): String?
    @JvmStatic external fun backNative(handle: Long): Boolean
    @JvmStatic external fun isPlayingNative(handle: Long): Boolean
    @JvmStatic external fun pollVideoNative(): String
    @JvmStatic external fun videoStatusNative(
        position: Double,
        duration: Double,
        paused: Boolean,
        muted: Boolean,
        hasFrame: Boolean,
        buffering: Float,
        error: String?,
    )
    @JvmStatic external fun focusNodeNative(handle: Long, node: Long)
    @JvmStatic external fun activateNodeNative(handle: Long, node: Long)
    @JvmStatic external fun pointerEventNative(handle: Long, action: Int, x: Float, y: Float)
    @JvmStatic external fun scrollNative(handle: Long, deltaY: Float)
    @JvmStatic external fun keyDownNative(handle: Long, keyCode: Int, shift: Int)
    @JvmStatic external fun focusedNodeNative(handle: Long): Long
    @JvmStatic external fun textInputFocusedNative(handle: Long): Boolean
    @JvmStatic external fun blurTextInputNative(handle: Long)
    @JvmStatic external fun textInputNative(handle: Long, text: String)
    @JvmStatic external fun focusedTextNative(handle: Long): String
    @JvmStatic external fun setFocusedTextNative(handle: Long, text: String)

    fun isNavigationKey(keyCode: Int): Boolean = when (keyCode) {
        KeyEvent.KEYCODE_DPAD_UP,
        KeyEvent.KEYCODE_DPAD_DOWN,
        KeyEvent.KEYCODE_DPAD_LEFT,
        KeyEvent.KEYCODE_DPAD_RIGHT,
        KeyEvent.KEYCODE_DPAD_CENTER,
        KeyEvent.KEYCODE_ENTER,
        KeyEvent.KEYCODE_BACK,
        KeyEvent.KEYCODE_ESCAPE,
        KeyEvent.KEYCODE_TAB,
        KeyEvent.KEYCODE_MENU,
        KeyEvent.KEYCODE_DEL,
        KeyEvent.KEYCODE_BUTTON_A,
        KeyEvent.KEYCODE_BUTTON_B,
        KeyEvent.KEYCODE_BUTTON_X,
        KeyEvent.KEYCODE_BUTTON_Y,
        KeyEvent.KEYCODE_BUTTON_START,
        KeyEvent.KEYCODE_BUTTON_SELECT -> true
        else -> false
    }
}

private const val POINTER_MOVE = 0
private const val POINTER_DOWN = 1
private const val POINTER_UP = 2
private const val HOST_VIEW_ID = -1
private const val MAX_ARTWORK_URLS = 96
private const val MAX_ARTWORK_BYTES = 16L * 1024L * 1024L
private val ARTWORK_URL_FIELDS = setOf(
    "artworkUrl",
    "backgroundUrl",
    "backdropUrl",
    "posterUrl",
    "logoUrl",
    "clearLogo",
    "profileAvatarUrl",
    "avatarUrl",
)
