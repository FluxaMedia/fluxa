package com.fluxa.app.ui.rust

import android.content.Context
import android.content.res.Configuration
import android.graphics.Rect
import android.os.Handler
import android.os.HandlerThread
import android.os.Build
import android.view.Choreographer
import android.view.HapticFeedbackConstants
import android.view.KeyEvent
import android.view.InputDevice
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeProvider
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsAnimationCompat
import androidx.core.view.WindowInsetsCompat
import org.json.JSONArray
import org.json.JSONObject

class FluxaNativeRendererView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    @Volatile private var nativeHandle: Long = 0L
    private var pendingFormFactor: String? = null
    private var lastCoreCommandJson: String? = null
    private var safeBottomInsetPx = 0
    private var safeTopInsetPx = 0
    private var accessSignature = ""
    private var accessNodes = emptyMap<Int, AccessNode>()
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
    private var pipState = 0
    private var canGoBack = true
    var onPipStateChanged: ((Int) -> Unit)? = null
    var onBackAvailable: ((Boolean) -> Unit)? = null
    var onAppIcon: ((String) -> Unit)? = null
    var onOpenUrl: ((String) -> Boolean)? = null
    var onExternalPlay: ((String) -> Unit)? = null
    var onInstallApk: ((String) -> Unit)? = null
    private var renderThread: HandlerThread? = null
    private var renderHandler: Handler? = null
    @Volatile private var rendering = false
    private val renderBusy = java.util.concurrent.atomic.AtomicBoolean(false)
    private val vsync = object : Choreographer.FrameCallback {
        override fun doFrame(frameTimeNanos: Long) {
            if (!rendering) return
            Choreographer.getInstance().postFrameCallback(this)
            if (renderBusy.compareAndSet(false, true)) renderHandler?.post(renderFrame)
        }
    }
    private val renderFrame = object : Runnable {
        override fun run() {
            try {
                renderOnce()
            } finally {
                renderBusy.set(false)
            }
        }
    }

    private fun renderOnce() {
        if (nativeHandle == 0L || !rendering) return
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
            post {
                val opened = onOpenUrl?.invoke(openUrl) ?: false
                if (!opened && nativeHandle != 0L) NativeRenderer.openUrlFailedNative(nativeHandle)
            }
        }
        val externalPlay = NativeRenderer.takeExternalPlayNative(nativeHandle)
        if (!externalPlay.isNullOrEmpty()) {
            post { onExternalPlay?.invoke(externalPlay) }
        }
        val installApk = NativeRenderer.takeInstallApkNative(nativeHandle)
        if (!installApk.isNullOrEmpty()) {
            post { onInstallApk?.invoke(installApk) }
        }
        val onPlayerScreen = NativeRenderer.wantsLandscapeNative(nativeHandle)
        if (onPlayerScreen != playerScreen) {
            playerScreen = onPlayerScreen
            applyFrameRate()
            post { onPlayerScreenChanged?.invoke(onPlayerScreen) }
        }
        val awake = NativeRenderer.keepAwakeNative(nativeHandle)
        if (awake != keepAwake) {
            keepAwake = awake
            post { keepScreenOn = awake }
        }
        val pip = NativeRenderer.pipStateNative(nativeHandle)
        if (pip != pipState) {
            pipState = pip
            post { onPipStateChanged?.invoke(pip) }
        }
        val backAvailable = NativeRenderer.canGoBackNative(nativeHandle)
        if (backAvailable != canGoBack) {
            canGoBack = backAvailable
            post { onBackAvailable?.invoke(backAvailable) }
        }
        val haptic = NativeRenderer.takeHapticNative()
        if (haptic != 0) post { performHaptic(haptic) }
        val videoRequests = NativeRenderer.pollVideoNative()
        if (videoRequests != "[]") {
            post { onVideoRequest?.invoke(videoRequests) }
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
            syncKeyboardInset(insets.getInsets(WindowInsetsCompat.Type.ime()).bottom)
            val imeVisible = insets.isVisible(WindowInsetsCompat.Type.ime())
            if (imeShown && !imeVisible && nativeHandle != 0L) {
                NativeRenderer.blurTextInputNative(nativeHandle)
            }
            imeShown = imeVisible
            insets
        }
        ViewCompat.setWindowInsetsAnimationCallback(
            this,
            object : WindowInsetsAnimationCompat.Callback(DISPATCH_MODE_CONTINUE_ON_SUBTREE) {
                override fun onProgress(
                    insets: WindowInsetsCompat,
                    runningAnimations: List<WindowInsetsAnimationCompat>,
                ): WindowInsetsCompat {
                    syncKeyboardInset(insets.getInsets(WindowInsetsCompat.Type.ime()).bottom)
                    return insets
                }
            },
        )
        isFocusable = true
        isFocusableInTouchMode = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
    }

    fun launcherFeed(): String? = if (nativeHandle == 0L) null else NativeRenderer.launcherFeedNative(nativeHandle)

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

    fun pressEnter() {
        if (nativeHandle != 0L) NativeRenderer.keyDownNative(nativeHandle, KeyEvent.KEYCODE_ENTER, 0)
    }

    fun pushAction(json: String) {
        val handle = nativeHandle
        if (handle == 0L) pendingActions += json else NativeRenderer.pushActionNative(handle, json)
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

    fun reportVideoStats(json: String) {
        NativeRenderer.videoStatsNative(json)
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
        syncFontScale()
        syncReduceMotion()
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

    private fun performHaptic(kind: Int) {
        val constant = when (kind) {
            1 -> HapticFeedbackConstants.LONG_PRESS
            2 -> if (Build.VERSION.SDK_INT >= 34) HapticFeedbackConstants.TOGGLE_ON else HapticFeedbackConstants.VIRTUAL_KEY
            else -> if (Build.VERSION.SDK_INT >= 34) HapticFeedbackConstants.TOGGLE_OFF else HapticFeedbackConstants.VIRTUAL_KEY
        }
        performHapticFeedback(constant)
    }

    private fun syncReduceMotion() {
        val scale = android.provider.Settings.Global.getFloat(
            context.contentResolver,
            android.provider.Settings.Global.ANIMATOR_DURATION_SCALE,
            1f,
        )
        NativeRenderer.setReduceMotionNative(scale == 0f)
    }

    private fun syncFontScale() {
        val handle = nativeHandle
        if (handle == 0L) return
        NativeRenderer.setFontScaleNative(handle, resources.configuration.fontScale)
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        syncFontScale()
    }

    private fun syncKeyboardInset(bottomPx: Int) {
        val handle = nativeHandle
        if (handle == 0L) return
        val density = resources.displayMetrics.density.coerceAtLeast(1f)
        NativeRenderer.setKeyboardInsetNative(handle, bottomPx / density)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        stopRenderThread()
        if (nativeHandle != 0L) NativeRenderer.surfaceDestroyedNative(nativeHandle)
    }

    private fun startRenderThread() {
        stopRenderThread()
        val thread = HandlerThread("fluxa-native-renderer").also { it.start() }
        renderThread = thread
        renderBusy.set(false)
        rendering = true
        applyFrameRate()
        renderHandler = Handler(thread.looper)
        Choreographer.getInstance().postFrameCallback(vsync)
    }

    private fun applyFrameRate() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
        val surface = holder.surface
        if (!surface.isValid) return
        val peak = display?.supportedModes?.maxOfOrNull { it.refreshRate } ?: 60f
        val rate = if (playerScreen) 60f else peak
        surface.setFrameRate(rate, Surface.FRAME_RATE_COMPATIBILITY_DEFAULT)
    }

    private fun stopRenderThread() {
        rendering = false
        Choreographer.getInstance().removeFrameCallback(vsync)
        renderHandler?.removeCallbacksAndMessages(null)
        renderHandler = null
        renderThread?.quitSafely()
        renderThread = null
    }

    override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
        super.onWindowFocusChanged(hasWindowFocus)
        if (hasWindowFocus) syncReduceMotion()
        val handle = nativeHandle
        if (handle != 0L) NativeRenderer.setWindowFocusedNative(handle, hasWindowFocus)
    }

    override fun onDetachedFromWindow() {
        textToolbar.close()
        if (NativeInputFocus.listener === inputFocusChanged) NativeInputFocus.listener = null
        imeSent = null
        boundField = 0L
        restartingFor = 0L
        stopRenderThread()
        if (nativeHandle != 0L) {
            NativeRenderer.destroyNative(nativeHandle)
            nativeHandle = 0L
        }
        super.onDetachedFromWindow()
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (nativeHandle == 0L) return false
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                requestFocus()
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_DOWN, logicalX(event.x), logicalY(event.y), event.eventTime)
            }
            MotionEvent.ACTION_MOVE -> {
                for (index in 0 until event.historySize) {
                    NativeRenderer.pointerEventNative(
                        nativeHandle,
                        POINTER_MOVE,
                        logicalX(event.getHistoricalX(index)),
                        logicalY(event.getHistoricalY(index)),
                        event.getHistoricalEventTime(index),
                    )
                }
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_MOVE, logicalX(event.x), logicalY(event.y), event.eventTime)
            }
            MotionEvent.ACTION_UP -> {
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_UP, logicalX(event.x), logicalY(event.y), event.eventTime)
                performClick()
                syncSoftKeyboard()
            }
            MotionEvent.ACTION_CANCEL -> {
                NativeRenderer.pointerEventNative(nativeHandle, POINTER_UP, logicalX(event.x), logicalY(event.y), event.eventTime)
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
                    NativeRenderer.pointerEventNative(handle, POINTER_MOVE, logicalX(event.x), logicalY(event.y), event.eventTime)
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
                    info.inputType = imeInputType(node.role.removeSuffix("_input"))
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
                    imeEdit(IME_SELECT_ALL)
                    imeEdit(IME_COMMIT, text?.toString().orEmpty(), 1)
                    publishIme(true)
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

    fun setPictureInPicture(on: Boolean) {
        if (nativeHandle != 0L) NativeRenderer.setPipNative(nativeHandle, on)
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
    private var boundField = 0L
    private var restartingFor = 0L
    private var imeSent: IntArray? = null
    var imeCursorUpdates = false
    private val inputMethod
        get() = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? android.view.inputmethod.InputMethodManager

    private val inputFocusChanged = Runnable { post { reconcileInput(0, false) } }

    internal fun imeField(): JSONObject? {
        val handle = nativeHandle
        if (handle == 0L) return null
        return NativeRenderer.imeFieldNative(handle)?.let { runCatching { JSONObject(it) }.getOrNull() }
    }

    private fun reconcileInput(settle: Int, show: Boolean) {
        val input = inputMethod ?: return
        val field = imeField()?.optLong("id") ?: 0L
        if (field == 0L) {
            if (boundField != 0L && settle < INPUT_SETTLE_FRAMES) {
                postOnAnimation { reconcileInput(settle + 1, show) }
                return
            }
            if (boundField != 0L) {
                boundField = 0L
                restartingFor = 0L
                imeSent = null
                input.hideSoftInputFromWindow(windowToken, 0)
            }
            return
        }
        if (field == boundField) {
            publishIme(false)
            if (show) input.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
            return
        }
        if (field == restartingFor) return
        restartingFor = field
        requestFocus()
        input.restartInput(this)
        input.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
    }

    internal fun bindIme(field: Long, state: IntArray?) {
        boundField = field
        restartingFor = 0L
        imeSent = state
        if (!toolbarPolling) {
            toolbarPolling = true
            postOnAnimation(toolbarPoll)
        }
    }

    private val textToolbar = TextToolbar(this)
    private var toolbarPolling = false

    private val toolbarPoll = object : Runnable {
        override fun run() {
            val handle = nativeHandle
            if (boundField == 0L || handle == 0L) {
                toolbarPolling = false
                textToolbar.close()
                return
            }
            textToolbar.sync(NativeRenderer.textToolbarNative(handle))
            postOnAnimation(this)
        }
    }

    internal fun dismissTextToolbar() {
        if (nativeHandle != 0L) NativeRenderer.textToolbarDismissNative(nativeHandle)
    }

    internal fun performTextAction(id: Int): Boolean {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as? android.content.ClipboardManager
        when (id) {
            android.R.id.selectAll -> imeEdit(IME_SELECT_ALL)
            android.R.id.copy -> {
                copySelection(clipboard)
                val end = NativeRenderer.imeStateNative(nativeHandle)?.get(1) ?: 0
                imeEdit(IME_SELECT, a = end, b = end)
            }
            android.R.id.cut -> {
                copySelection(clipboard)
                imeEdit(IME_COMMIT, "", 1)
            }
            android.R.id.paste, android.R.id.pasteAsPlainText -> {
                val text = clipboard?.primaryClip?.getItemAt(0)?.coerceToText(context)?.toString().orEmpty()
                if (text.isNotEmpty()) imeEdit(IME_COMMIT, text, 1)
            }
            else -> return false
        }
        publishIme(true)
        return true
    }

    private fun copySelection(clipboard: android.content.ClipboardManager?) {
        val selected = imeText(2, 0)
        if (selected.isNotEmpty()) {
            clipboard?.setPrimaryClip(android.content.ClipData.newPlainText(null, selected))
        }
    }

    internal fun publishIme(fromIme: Boolean) {
        val handle = nativeHandle
        val input = inputMethod ?: return
        if (handle == 0L) return
        val state = NativeRenderer.imeStateNative(handle) ?: return
        val last = imeSent
        imeSent = state
        when {
            fromIme -> input.updateSelection(this, state[0], state[1], state[2], state[3])
            last == null -> Unit
            last[4] != state[4] -> input.restartInput(this)
            !state.contentEquals(last) -> input.updateSelection(this, state[0], state[1], state[2], state[3])
        }
        if (imeCursorUpdates) sendCursorAnchor(state)
    }

    private fun sendCursorAnchor(state: IntArray) {
        val input = inputMethod ?: return
        val box = NativeRenderer.imeCaretNative(nativeHandle) ?: return
        val density = resources.displayMetrics.density
        val left = box[0] * density
        val top = box[1] * density
        val bottom = (box[1] + box[3]) * density
        val builder = android.view.inputmethod.CursorAnchorInfo.Builder()
            .setSelectionRange(state[0], state[1])
            .setMatrix(android.graphics.Matrix())
            .setInsertionMarkerLocation(left, top, bottom, bottom, android.view.inputmethod.CursorAnchorInfo.FLAG_HAS_VISIBLE_REGION)
        if (state[2] >= 0) {
            val text = NativeRenderer.imeTextNative(nativeHandle, 3, 0)
            val from = state[2].coerceIn(0, text.length)
            val to = state[3].coerceIn(from, text.length)
            builder.setComposingText(from, text.subSequence(from, to))
        }
        input.updateCursorAnchorInfo(this, builder.build())
    }

    internal fun imeText(mode: Int, count: Int): String = NativeRenderer.imeTextNative(nativeHandle, mode, count)

    internal fun imeEdit(op: Int, text: String = "", a: Int = 0, b: Int = 0): Boolean =
        nativeHandle != 0L && NativeRenderer.imeEditNative(nativeHandle, op, text, a, b)

    private fun syncSoftKeyboard() = post { reconcileInput(0, true) }

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

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? {
        val handle = nativeHandle
        val field = imeField() ?: return null
        val kind = field.optString("kind")
        outAttrs.fieldId = field.optLong("id").let { (it xor (it ushr 32)).toInt() }
        outAttrs.fieldName = field.optString("hint")
        outAttrs.inputType = imeInputType(kind)
        outAttrs.imeOptions = imeOptions(kind)
        if (android.os.Build.VERSION.SDK_INT >= 26) outAttrs.hintText = field.optString("hint")
        val state = NativeRenderer.imeStateNative(handle)
        outAttrs.initialSelStart = state?.get(0) ?: 0
        outAttrs.initialSelEnd = state?.get(1) ?: 0
        bindIme(field.optLong("id"), state)
        return NativeInputConnection(this, kind == "multiline")
    }
}

private var jvmCrashReporterInstalled = false

fun installJvmCrashReporter() {
    if (jvmCrashReporterInstalled) return
    jvmCrashReporterInstalled = true
    val previous = Thread.getDefaultUncaughtExceptionHandler()
    Thread.setDefaultUncaughtExceptionHandler { thread, error ->
        runCatching { NativeRenderer.reportJvmCrashNative(android.util.Log.getStackTraceString(error)) }
        previous?.uncaughtException(thread, error)
    }
}

private object NativeRenderer {
    init {
        System.loadLibrary("fluxa_android_renderer")
    }

    @JvmStatic external fun createNative(density: Float, artworkCacheDir: String): Long
    @JvmStatic external fun reportJvmCrashNative(trace: String)
    @JvmStatic external fun setSafeBottomInsetNative(handle: Long, insetDp: Float)

    @JvmStatic external fun takeHapticNative(): Int
    @JvmStatic external fun setReduceMotionNative(reduced: Boolean)
    @JvmStatic external fun setFontScaleNative(handle: Long, scale: Float)
    @JvmStatic external fun setKeyboardInsetNative(handle: Long, insetDp: Float)
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
    @JvmStatic external fun openUrlFailedNative(handle: Long)
    @JvmStatic external fun takeExternalPlayNative(handle: Long): String?
    @JvmStatic external fun takeInstallApkNative(handle: Long): String?
    @JvmStatic external fun launcherFeedNative(handle: Long): String?
    @JvmStatic external fun canGoBackNative(handle: Long): Boolean
    @JvmStatic external fun backNative(handle: Long): Boolean
    @JvmStatic external fun isPlayingNative(handle: Long): Boolean
    @JvmStatic external fun keepAwakeNative(handle: Long): Boolean
    @JvmStatic external fun pipStateNative(handle: Long): Int
    @JvmStatic external fun setPipNative(handle: Long, on: Boolean)
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
    @JvmStatic external fun videoStatsNative(json: String)
    @JvmStatic external fun videoChaptersNative(json: String)
    @JvmStatic external fun videoThumbnailNative(data: ByteArray)
    @JvmStatic external fun videoThumbnailProgressNative(done: Int, count: Int)
    @JvmStatic external fun accessibilityTreeNative(handle: Long): String?
    @JvmStatic external fun focusNodeNative(handle: Long, node: Long)
    @JvmStatic external fun activateNodeNative(handle: Long, node: Long)
    @JvmStatic external fun pointerEventNative(handle: Long, action: Int, x: Float, y: Float, eventMs: Long)
    @JvmStatic external fun scrollNative(handle: Long, deltaY: Float)
    @JvmStatic external fun keyDownNative(handle: Long, keyCode: Int, shift: Int)
    @JvmStatic external fun focusedNodeNative(handle: Long): Long
    @JvmStatic external fun textInputFocusedNative(handle: Long): Boolean
    @JvmStatic external fun blurTextInputNative(handle: Long)
    @JvmStatic external fun textInputNative(handle: Long, text: String)
    @JvmStatic external fun imeFieldNative(handle: Long): String?
    @JvmStatic external fun imeStateNative(handle: Long): IntArray?
    @JvmStatic external fun imeCaretNative(handle: Long): FloatArray?
    @JvmStatic external fun textToolbarNative(handle: Long): FloatArray?
    @JvmStatic external fun textToolbarDismissNative(handle: Long)
    @JvmStatic external fun imeTextNative(handle: Long, mode: Int, count: Int): String
    @JvmStatic external fun imeEditNative(handle: Long, op: Int, text: String, a: Int, b: Int): Boolean

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
private const val INPUT_SETTLE_FRAMES = 3
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
