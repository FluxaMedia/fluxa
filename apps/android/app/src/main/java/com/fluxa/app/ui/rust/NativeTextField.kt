package com.fluxa.app.ui.rust

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.os.SystemClock
import android.text.Editable
import android.text.InputFilter
import android.text.InputType
import android.text.TextWatcher
import android.util.TypedValue
import android.view.Gravity
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.FrameLayout
import org.json.JSONObject

internal class NativeTextField(
    context: Context,
    private val renderer: FluxaNativeRendererView,
) : EditText(context) {
    private var field = 0L
    private var applying = false
    private var editedAt = 0L
    private var bounds = IntArray(4)

    private val tick = object : Runnable {
        override fun run() {
            sync()
            postOnAnimation(this)
        }
    }

    init {
        background = null
        setPadding(0, 0, 0, 0)
        includeFontPadding = false
        gravity = Gravity.CENTER_VERTICAL or Gravity.START
        setTextColor(Color.WHITE)
        setHintTextColor(Color.argb(140, 255, 255, 255))
        highlightColor = Color.argb(90, 255, 255, 255)
        visibility = GONE
        addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(text: CharSequence?, start: Int, count: Int, after: Int) = Unit

            override fun onTextChanged(text: CharSequence?, start: Int, before: Int, count: Int) = Unit

            override fun afterTextChanged(text: Editable?) {
                if (applying || field == 0L) return
                editedAt = SystemClock.uptimeMillis()
                renderer.setFieldText(field, text?.toString().orEmpty())
            }
        })
        setOnEditorActionListener { _, _, _ ->
            renderer.pressEnter()
            true
        }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        postOnAnimation(tick)
    }

    override fun onDetachedFromWindow() {
        removeCallbacks(tick)
        super.onDetachedFromWindow()
    }

    private fun sync() {
        val json = renderer.textFieldJson()?.let { runCatching { JSONObject(it) }.getOrNull() }
        if (json == null) {
            hide()
            return
        }
        val density = resources.displayMetrics.density
        val next = intArrayOf(
            (json.getDouble("x") * density).toInt(),
            (json.getDouble("y") * density).toInt(),
            (json.getDouble("width") * density).toInt(),
            (json.getDouble("height") * density).toInt(),
        )
        if (!next.contentEquals(bounds)) {
            bounds = next
            layoutParams = FrameLayout.LayoutParams(next[2], next[3]).apply {
                leftMargin = next[0]
                topMargin = next[1]
            }
        }
        val id = json.getLong("id")
        val text = json.optString("text")
        if (id != field) {
            applying = true
            field = id
            configure(json.optString("kind"), json.optDouble("size", 0.0).toFloat())
            apply(text)
            visibility = VISIBLE
            requestFocus()
            post { keyboard()?.showSoftInput(this, InputMethodManager.SHOW_IMPLICIT) }
        } else if (text != getText().toString() && SystemClock.uptimeMillis() - editedAt > SETTLE_MS) {
            apply(text)
        }
    }

    private fun configure(kind: String, size: Float) {
        val code = kind == "code"
        setTextSize(TypedValue.COMPLEX_UNIT_PX, (if (size > 0f) size else 16f) * resources.displayMetrics.density)
        typeface = if (code) Typeface.DEFAULT_BOLD else Typeface.DEFAULT
        letterSpacing = if (code) 0.45f else 0f
        filters = if (code) arrayOf(InputFilter.AllCaps(), InputFilter.LengthFilter(CODE_LENGTH)) else emptyArray()
        isSingleLine = kind != "multiline"
        inputType = when (kind) {
            "password" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
            "number" -> InputType.TYPE_CLASS_NUMBER
            "url" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
            "multiline" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE
            "code" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_CAP_CHARACTERS or
                InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            else -> InputType.TYPE_CLASS_TEXT
        }
        imeOptions = when (kind) {
            "search" -> EditorInfo.IME_ACTION_SEARCH
            "multiline" -> EditorInfo.IME_ACTION_NONE
            else -> EditorInfo.IME_ACTION_DONE
        }
        keyboard()?.restartInput(this)
    }

    private fun apply(text: String) {
        applying = true
        setText(text)
        setSelection(length())
        applying = false
    }

    private fun hide() {
        if (field == 0L) return
        field = 0L
        clearFocus()
        visibility = GONE
        keyboard()?.hideSoftInputFromWindow(windowToken, 0)
    }

    private fun keyboard() = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
}

private const val SETTLE_MS = 500L
private const val CODE_LENGTH = 6
