package com.fluxa.app.ui.rust

import android.content.Context
import android.graphics.Rect
import android.graphics.RectF
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
    private var safeTopInsetPx = 0
    private var accessSignature = ""
    private var accessNodes = emptyMap<Int, AccessNode>()
    private var accessCaret: RectF? = null
    private var accessSelection: IntArray? = null
    private var accessFocused = -1
    var sessionDataDir: String? = null
    var legacyProfilesJson: (() -> String)? = null
    private val pendingActions = mutableListOf<String>()
    var onNativeAction: ((String) -> Unit)? = null
    var onCoreCommand: ((String) -> Unit)? = null
    var onVideoRequest: ((String) -> Unit)? = null
    var onPlayerScreenChanged: ((Boolean) -> Unit)? = null
    private var playerScreen = false
    private var keepAwake = false
    var onAppIcon: ((String) -> Unit)? = null
    var onOpenUrl: ((String) -> Unit)? = null
    var onInstallApk: ((String) -> Unit)? = null
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
            val installApk = NativeRenderer.takeInstallApkNative(nativeHandle)
            if (!installApk.isNullOrEmpty()) {
                post { onInstallApk?.invoke(installApk) }
            }
            val onPlayerScreen = NativeRenderer.wantsLandscapeNative(nativeHandle)
            if (onPlayerScreen != playerScreen) {
                playerScreen = onPlayerScreen
                post { onPlayerScreenChanged?.invoke(onPlayerScreen) }
            }
            val awake = NativeRenderer.keepAwakeNative(nativeHandle)
            if (awake != keepAwake) {
                keepAwake = awake
                post { keepScreenOn = awake }
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
            safeTopInsetPx = insets
                .getInsets(WindowInsetsCompat.Type.statusBars() or WindowInsetsCompat.Type.displayCutout())
                .top
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

    fun launcherFeed(): String? = if (nativeHandle == 0L) null else NativeRenderer.launcherFeedNative(nativeHandle)

    /** Pushes the platform data-source projection into the Rust-owned scene. */
    fun setHomeStateJson(json: String) {
        pendingHomeStateJson = json
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

    fun reportVideoTracks(json: String) {
        NativeRenderer.videoTracksNative(json)
    }

    fun reportVideoThumbnail(data: ByteArray) {
        NativeRenderer.videoThumbnailNative(data)
    }

    fun reportVideoThumbnailProgress(done: Int, count: Int) {
        NativeRenderer.videoThumbnailProgressNative(done, count)
    }

    fun reportVideoChapters(json: String) {
        NativeRenderer.videoChaptersNative(json)
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
        NativeRenderer.setSafeTopInsetNative(handle, safeTopInsetPx / density)
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

    override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
        super.onWindowFocusChanged(hasWindowFocus)
        val handle = nativeHandle
        if (handle != 0L) NativeRenderer.setWindowFocusedNative(handle, hasWindowFocus)
    }

    override fun onDetachedFromWindow() {
        if (NativeInputFocus.listener === inputFocusChanged) NativeInputFocus.listener = null
        inputEditable = null
        boundField = 0L
        restartingFor = 0L
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

    private val accessibilityManager =
        context.getSystemService(Context.ACCESSIBILITY_SERVICE) as android.view.accessibility.AccessibilityManager

    private val accessibilityPoll = object : Runnable {
        override fun run() {
            refreshAccessibility()
            if (accessibilityManager.isEnabled) postDelayed(this, ACCESSIBILITY_POLL_MS)
        }
    }

    private val accessibilityStateListener =
        android.view.accessibility.AccessibilityManager.AccessibilityStateChangeListener { enabled ->
            removeCallbacks(accessibilityPoll)
            if (enabled) post(accessibilityPoll)
        }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        NativeInputFocus.listener = inputFocusChanged
        accessibilityManager.addAccessibilityStateChangeListener(accessibilityStateListener)
        if (accessibilityManager.isEnabled) post(accessibilityPoll)
    }

    private fun loadAccessTree(): String? {
        val handle = nativeHandle
        return if (handle == 0L) null else NativeRenderer.accessibilityTreeNative(handle)
    }

    private fun refreshAccessibility() {
        val json = loadAccessTree() ?: return
        if (json == accessSignature) return
        accessSignature = json
        val previousFocus = accessFocused
        val previousNodes = accessNodes
        val tree = runCatching { JSONObject(json) }.getOrNull() ?: return
        val density = resources.displayMetrics.density
        val nodes = LinkedHashMap<Int, AccessNode>()
        val array = tree.optJSONArray("nodes") ?: JSONArray()
        for (index in 0 until array.length()) {
            val node = AccessNode.parse(array.getJSONObject(index), density)
            nodes[node.id] = node
        }
        accessNodes = nodes
        accessCaret = tree.optJSONArray("caret")?.let {
            RectF(
                (it.getDouble(0) * density).toFloat(),
                (it.getDouble(1) * density).toFloat(),
                ((it.getDouble(0) + it.getDouble(2)) * density).toFloat(),
                ((it.getDouble(1) + it.getDouble(3)) * density).toFloat(),
            )
        }
        accessSelection = tree.optJSONArray("selection")?.let { intArrayOf(it.getInt(0), it.getInt(1)) }
        accessFocused = if (tree.isNull("focused")) -1 else tree.optInt("focused", -1)
        if (!accessibilityManager.isEnabled) return
        if (previousNodes.keys != nodes.keys) {
            val event = AccessibilityEvent.obtain(AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED)
            event.contentChangeTypes = AccessibilityEvent.CONTENT_CHANGE_TYPE_SUBTREE
            event.setSource(this, HOST_VIEW_ID)
            parent?.requestSendAccessibilityEvent(this, event)
        } else {
            for ((id, node) in nodes) {
                if (previousNodes[id]?.describes(node) == false) notifyNodeChanged(id)
            }
        }
        if (accessFocused != previousFocus && accessFocused != -1) {
            val event = AccessibilityEvent.obtain(AccessibilityEvent.TYPE_VIEW_FOCUSED)
            event.setSource(this, accessFocused)
            parent?.requestSendAccessibilityEvent(this, event)
        }
    }

    private fun notifyNodeChanged(id: Int) {
        val event = AccessibilityEvent.obtain(AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED)
        event.contentChangeTypes = AccessibilityEvent.CONTENT_CHANGE_TYPE_CONTENT_DESCRIPTION or
            AccessibilityEvent.CONTENT_CHANGE_TYPE_TEXT
        event.setSource(this, id)
        parent?.requestSendAccessibilityEvent(this, event)
    }

    private fun seekBy(seconds: Double): Boolean {
        val slider = accessNodes.values.firstOrNull { it.role == "slider" } ?: return false
        val range = slider.range ?: return false
        return seekTo((range[2] + seconds).coerceIn(range[0], range[1]))
    }

    private fun seekTo(seconds: Double): Boolean {
        if (nativeHandle == 0L) return false
        NativeRenderer.pushActionNative(
            nativeHandle,
            JSONObject(mapOf("type" to "mediaCommand", "command" to "seekTo", "value" to seconds)).toString(),
        )
        return true
    }

    private val accessibilityProvider = object : AccessibilityNodeProvider() {
        override fun createAccessibilityNodeInfo(virtualViewId: Int): AccessibilityNodeInfo? {
            if (virtualViewId == HOST_VIEW_ID) {
                return AccessibilityNodeInfo.obtain(this@FluxaNativeRendererView).apply {
                    onInitializeAccessibilityNodeInfo(this)
                    for (id in accessNodes.keys) addChild(this@FluxaNativeRendererView, id)
                }
            }
            val node = accessNodes[virtualViewId] ?: return null
            val info = AccessibilityNodeInfo.obtain()
            info.packageName = context.packageName
            info.setSource(this@FluxaNativeRendererView, virtualViewId)
            info.setParent(this@FluxaNativeRendererView, HOST_VIEW_ID)
            info.isEnabled = true
            info.isFocusable = true
            info.isFocused = virtualViewId == accessFocused
            info.isVisibleToUser = visibility == VISIBLE && node.visible(width, height)
            info.isSelected = node.selected
            info.isAccessibilityFocused = virtualViewId == accessibilityFocusedNode
            info.setBoundsInParent(node.bounds)
            val screen = IntArray(2).also { getLocationOnScreen(it) }
            info.setBoundsInScreen(Rect(node.bounds).apply { offset(screen[0], screen[1]) })
            if (!node.isStatic()) info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_FOCUS)
            info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_ACCESSIBILITY_FOCUS)
            if (virtualViewId == accessibilityFocusedNode) {
                info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLEAR_ACCESSIBILITY_FOCUS)
            }
            when (node.role) {
                "switch" -> {
                    info.className = "android.widget.Switch"
                    info.isCheckable = true
                    info.isChecked = node.checked == true
                    info.text = node.label
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
                }
                "slider" -> {
                    info.className = "android.widget.SeekBar"
                    info.contentDescription = node.label
                    node.range?.let {
                        info.rangeInfo = AccessibilityNodeInfo.RangeInfo.obtain(
                            AccessibilityNodeInfo.RangeInfo.RANGE_TYPE_FLOAT,
                            it[0].toFloat(),
                            it[1].toFloat(),
                            it[2].toFloat(),
                        )
                    }
                    if (android.os.Build.VERSION.SDK_INT >= 30) info.stateDescription = node.value
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_PROGRESS)
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_FORWARD)
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_BACKWARD)
                }
                "text", "heading" -> {
                    info.className = "android.widget.TextView"
                    info.text = node.label
                    if (node.role == "heading" && android.os.Build.VERSION.SDK_INT >= 28) info.isHeading = true
                }
                "text_input", "search_input", "password_input", "url_input", "number_input", "multiline_input" -> {
                    info.className = "android.widget.EditText"
                    info.isEditable = true
                    info.isMultiLine = node.role == "multiline_input"
                    info.isPassword = node.role == "password_input"
                    info.text = node.value ?: ""
                    if (android.os.Build.VERSION.SDK_INT >= 26) info.hintText = node.label
                    info.contentDescription = node.label
                    info.inputType = node.inputType()
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_TEXT)
                }
                else -> {
                    info.className = "android.widget.Button"
                    info.text = node.label
                    if (node.value != null && android.os.Build.VERSION.SDK_INT >= 30) {
                        info.stateDescription = node.value
                    }
                    info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
                }
            }
            return info
        }

        override fun performAction(virtualViewId: Int, action: Int, arguments: android.os.Bundle?): Boolean {
            if (virtualViewId == HOST_VIEW_ID) return false
            val node = accessNodes[virtualViewId]
            if (node == null || nativeHandle == 0L) return false
            return when (action) {
                AccessibilityNodeInfo.ACTION_CLICK -> {
                    if (node.isTextInput()) {
                        NativeRenderer.focusNodeNative(nativeHandle, virtualViewId.toLong())
                        syncSoftKeyboard()
                    } else {
                        NativeRenderer.activateNodeNative(nativeHandle, virtualViewId.toLong())
                    }
                    true
                }
                AccessibilityNodeInfo.ACTION_FOCUS -> {
                    NativeRenderer.focusNodeNative(nativeHandle, virtualViewId.toLong())
                    true
                }
                AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS -> {
                    accessibilityFocusedNode = virtualViewId
                    NativeRenderer.focusNodeNative(nativeHandle, virtualViewId.toLong())
                    notifyNodeChanged(virtualViewId)
                    true
                }
                AccessibilityNodeInfo.ACTION_CLEAR_ACCESSIBILITY_FOCUS -> {
                    if (accessibilityFocusedNode == virtualViewId) accessibilityFocusedNode = -1
                    true
                }
                AccessibilityNodeInfo.ACTION_SET_TEXT -> {
                    val text = arguments?.getCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE)
                    NativeRenderer.focusNodeNative(nativeHandle, virtualViewId.toLong())
                    NativeRenderer.setFieldTextNative(nativeHandle, NativeRenderer.focusedFieldIdNative(nativeHandle), text?.toString() ?: "")
                    true
                }
                AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_PROGRESS.id ->
                    node.role == "slider" && seekTo(
                        arguments?.getFloat(AccessibilityNodeInfo.ACTION_ARGUMENT_PROGRESS_VALUE)?.toDouble() ?: return false,
                    )
                AccessibilityNodeInfo.ACTION_SCROLL_FORWARD -> node.role == "slider" && seekBy(SEEK_STEP_SECONDS)
                AccessibilityNodeInfo.ACTION_SCROLL_BACKWARD -> node.role == "slider" && seekBy(-SEEK_STEP_SECONDS)
                else -> false
            }
        }

        override fun findFocus(focus: Int): AccessibilityNodeInfo? {
            val id = if (focus == AccessibilityNodeInfo.FOCUS_ACCESSIBILITY) accessibilityFocusedNode else accessFocused
            return if (id == -1) null else createAccessibilityNodeInfo(id)
        }
    }

    private var accessibilityFocusedNode = -1

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
    private var inputEditable: android.text.Editable? = null
    private var lastLocalEditAt = 0L
    private var boundField = 0L
    private var restartingFor = 0L
    private val inputMethod
        get() = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? android.view.inputmethod.InputMethodManager

    private val inputFocusChanged = Runnable { post { reconcileInput(0) } }

    private fun reconcileInput(settle: Int) {
        val input = inputMethod ?: return
        val field = if (NativeRenderer.textInputFocusedNative(nativeHandle)) {
            NativeRenderer.focusedFieldIdNative(nativeHandle)
        } else {
            0L
        }
        if (field == 0L) {
            if (boundField != 0L && settle < INPUT_SETTLE_FRAMES) {
                postOnAnimation { reconcileInput(settle + 1) }
                return
            }
            if (boundField != 0L) {
                boundField = 0L
                restartingFor = 0L
                inputEditable = null
                input.hideSoftInputFromWindow(windowToken, 0)
            }
            return
        }
        if (field == boundField) {
            input.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
            return
        }
        if (field == restartingFor) return
        restartingFor = field
        inputEditable?.let {
            BaseInputConnection.removeComposingSpans(it)
            input.updateSelection(this, it.length, it.length, -1, -1)
        }
        requestFocus()
        input.restartInput(this)
        input.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
    }

    private val editableSync = object : Runnable {
        override fun run() {
            val editable = inputEditable ?: return
            if (nativeHandle == 0L || NativeRenderer.focusedFieldIdNative(nativeHandle) != boundField) {
                return
            }
            val current = NativeRenderer.focusedTextNative(nativeHandle)
            val settled = android.os.SystemClock.uptimeMillis() - lastLocalEditAt > LOCAL_EDIT_SETTLE_MS
            if (settled && current != editable.toString()) {
                editable.replace(0, editable.length, current)
                val selection = accessSelection
                val start = (selection?.get(0) ?: editable.length).coerceIn(0, editable.length)
                val end = (selection?.get(1) ?: editable.length).coerceIn(0, editable.length)
                android.text.Selection.setSelection(editable, start, end)
                reportSelection(editable)
            }
            postDelayed(this, EDITABLE_POLL_MS)
        }
    }

    private fun reportSelection(editable: android.text.Editable) {
        val input = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? android.view.inputmethod.InputMethodManager ?: return
        val start = android.text.Selection.getSelectionStart(editable)
        val end = android.text.Selection.getSelectionEnd(editable)
        val composingStart = BaseInputConnection.getComposingSpanStart(editable)
        val composingEnd = BaseInputConnection.getComposingSpanEnd(editable)
        input.updateSelection(this, start, end, composingStart, composingEnd)
        val caret = accessCaret
        val bounds = accessNodes[accessFocused]?.bounds ?: return
        val top = caret?.top ?: bounds.top.toFloat()
        val bottom = caret?.bottom ?: bounds.bottom.toFloat()
        val x = caret?.left ?: bounds.left.toFloat()
        val anchor = android.view.inputmethod.CursorAnchorInfo.Builder()
            .setSelectionRange(start, end)
            .setMatrix(android.graphics.Matrix())
        if (composingStart >= 0 && composingEnd > composingStart) {
            anchor.setComposingText(composingStart, editable.subSequence(composingStart, composingEnd))
        }
        anchor.setInsertionMarkerLocation(x, top, bottom, bottom, 0)
        input.updateCursorAnchorInfo(this, anchor.build())
    }

    private fun syncSoftKeyboard() = post { reconcileInput(0) }

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

    override fun onCheckIsTextEditor(): Boolean =
        nativeHandle != 0L && NativeRenderer.textInputFocusedNative(nativeHandle)

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        refreshAccessibility()
        val focusedNode = accessNodes[accessFocused]?.takeIf { it.isTextInput() }
        val field = if (nativeHandle != 0L) NativeRenderer.focusedFieldIdNative(nativeHandle) else 0L
        outAttrs.fieldId = (field xor (field ushr 32)).toInt()
        outAttrs.fieldName = focusedNode?.label
        outAttrs.inputType = focusedNode?.inputType() ?: android.text.InputType.TYPE_CLASS_TEXT
        outAttrs.imeOptions = when (focusedNode?.role) {
            "search_input" -> EditorInfo.IME_ACTION_SEARCH
            "multiline_input" -> EditorInfo.IME_ACTION_NONE
            else -> EditorInfo.IME_ACTION_DONE
        }
        if (android.os.Build.VERSION.SDK_INT >= 26) focusedNode?.label?.let { outAttrs.hintText = it }
        val connection = object : BaseInputConnection(this, true) {
            private fun sync(result: Boolean): Boolean {
                if (nativeHandle != 0L) {
                    lastLocalEditAt = android.os.SystemClock.uptimeMillis()
                    NativeRenderer.setFieldTextNative(nativeHandle, field, editable.toString())
                }
                if (batch == 0) editable?.let { reportSelection(it) }
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
            boundField = field
            restartingFor = 0L
            inputEditable = connection.editable
            removeCallbacks(editableSync)
            postDelayed(editableSync, EDITABLE_POLL_MS)
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
    @JvmStatic external fun setSafeTopInsetNative(handle: Long, insetDp: Float)
    @JvmStatic external fun setWindowFocusedNative(handle: Long, focused: Boolean)
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
    @JvmStatic external fun takeInstallApkNative(handle: Long): String?
    @JvmStatic external fun launcherFeedNative(handle: Long): String?
    @JvmStatic external fun backNative(handle: Long): Boolean
    @JvmStatic external fun isPlayingNative(handle: Long): Boolean
    @JvmStatic external fun keepAwakeNative(handle: Long): Boolean
    @JvmStatic external fun wantsLandscapeNative(handle: Long): Boolean
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
    @JvmStatic external fun videoTracksNative(json: String)
    @JvmStatic external fun videoChaptersNative(json: String)
    @JvmStatic external fun videoThumbnailNative(data: ByteArray)
    @JvmStatic external fun videoThumbnailProgressNative(done: Int, count: Int)
    @JvmStatic external fun accessibilityTreeNative(handle: Long): String?
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
    @JvmStatic external fun setFieldTextNative(handle: Long, field: Long, text: String)
    @JvmStatic external fun focusedFieldIdNative(handle: Long): Long

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
private const val ACCESSIBILITY_POLL_MS = 250L
private const val EDITABLE_POLL_MS = 200L
private const val INPUT_SETTLE_FRAMES = 3
private const val LOCAL_EDIT_SETTLE_MS = 500L
private const val SEEK_STEP_SECONDS = 10.0

private class AccessNode(
    val id: Int,
    val role: String,
    val label: String,
    val value: String?,
    val checked: Boolean?,
    val selected: Boolean,
    val bounds: Rect,
    val range: DoubleArray?,
) {
    fun isTextInput() = role.endsWith("_input")

    fun isStatic() = role == "text" || role == "heading"

    fun visible(width: Int, height: Int) = bounds.right > 0 && bounds.bottom > 0 && bounds.left < width && bounds.top < height

    fun describes(other: AccessNode) = label == other.label && value == other.value &&
        checked == other.checked && selected == other.selected

    fun inputType(): Int = when (role) {
        "password_input" -> android.text.InputType.TYPE_CLASS_TEXT or android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD
        "url_input" -> android.text.InputType.TYPE_CLASS_TEXT or android.text.InputType.TYPE_TEXT_VARIATION_URI
        "number_input" -> android.text.InputType.TYPE_CLASS_NUMBER
        "multiline_input" -> android.text.InputType.TYPE_CLASS_TEXT or android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE
        else -> android.text.InputType.TYPE_CLASS_TEXT
    }

    companion object {
        fun parse(json: JSONObject, density: Float): AccessNode {
            val box = json.getJSONArray("bounds")
            val left = (box.getDouble(0) * density).toInt()
            val top = (box.getDouble(1) * density).toInt()
            val range = json.optJSONArray("range")?.let {
                doubleArrayOf(it.getDouble(0), it.getDouble(1), it.getDouble(2))
            }
            return AccessNode(
                id = json.getInt("id"),
                role = json.getString("role"),
                label = json.optString("label"),
                value = if (json.isNull("value")) null else json.getString("value"),
                checked = if (json.isNull("checked")) null else json.getBoolean("checked"),
                selected = json.optBoolean("selected"),
                bounds = Rect(
                    left,
                    top,
                    left + (box.getDouble(2) * density).toInt(),
                    top + (box.getDouble(3) * density).toInt(),
                ),
                range = range,
            )
        }
    }
}
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
