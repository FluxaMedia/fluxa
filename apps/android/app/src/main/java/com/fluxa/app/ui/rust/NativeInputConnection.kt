package com.fluxa.app.ui.rust

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.text.InputType
import android.view.KeyEvent
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.ExtractedText
import android.view.inputmethod.ExtractedTextRequest
import android.view.inputmethod.InputConnection

internal const val IME_COMMIT = 0
internal const val IME_COMPOSE = 1
internal const val IME_COMPOSE_REGION = 2
internal const val IME_FINISH = 3
internal const val IME_DELETE = 4
internal const val IME_DELETE_CODE_POINTS = 5
internal const val IME_SELECT = 6
internal const val IME_SELECT_ALL = 7
internal const val IME_BACKSPACE = 8

internal fun imeInputType(kind: String): Int = when (kind) {
    "password" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
    "number" -> InputType.TYPE_CLASS_NUMBER
    "pin" -> InputType.TYPE_CLASS_NUMBER or InputType.TYPE_NUMBER_VARIATION_PASSWORD
    "url" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
    "multiline" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE or
        InputType.TYPE_TEXT_FLAG_CAP_SENTENCES
    "code" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_CAP_CHARACTERS or
        InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
    else -> InputType.TYPE_CLASS_TEXT
}

internal fun imeOptions(kind: String): Int {
    val action = when (kind) {
        "search" -> EditorInfo.IME_ACTION_SEARCH
        "multiline" -> EditorInfo.IME_ACTION_NONE
        else -> EditorInfo.IME_ACTION_DONE
    }
    return action or EditorInfo.IME_FLAG_NO_EXTRACT_UI or EditorInfo.IME_FLAG_NO_FULLSCREEN
}

internal class NativeInputConnection(
    private val view: FluxaNativeRendererView,
    private val multiline: Boolean,
) : BaseInputConnection(view, true) {
    private var batch = 0

    private val clipboard
        get() = view.context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager

    private fun edit(op: Int, text: String = "", a: Int = 0, b: Int = 0): Boolean {
        val done = view.imeEdit(op, text, a, b)
        if (batch == 0) view.publishIme(true)
        return done
    }

    override fun getTextBeforeCursor(length: Int, flags: Int): CharSequence = view.imeText(0, length)

    override fun getTextAfterCursor(length: Int, flags: Int): CharSequence = view.imeText(1, length)

    override fun getSelectedText(flags: Int): CharSequence? = view.imeText(2, 0).ifEmpty { null }

    override fun getCursorCapsMode(reqModes: Int): Int = 0

    override fun getExtractedText(request: ExtractedTextRequest?, flags: Int): ExtractedText? = null

    override fun commitText(text: CharSequence, newCursorPosition: Int) =
        edit(IME_COMMIT, text.toString(), newCursorPosition)

    override fun setComposingText(text: CharSequence, newCursorPosition: Int) =
        edit(IME_COMPOSE, text.toString(), newCursorPosition)

    override fun setComposingRegion(start: Int, end: Int) = edit(IME_COMPOSE_REGION, a = start, b = end)

    override fun finishComposingText() = edit(IME_FINISH)

    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int) =
        edit(IME_DELETE, a = beforeLength, b = afterLength)

    override fun deleteSurroundingTextInCodePoints(beforeLength: Int, afterLength: Int) =
        edit(IME_DELETE_CODE_POINTS, a = beforeLength, b = afterLength)

    override fun setSelection(start: Int, end: Int) = edit(IME_SELECT, a = start, b = end)

    override fun beginBatchEdit(): Boolean {
        batch++
        return true
    }

    override fun endBatchEdit(): Boolean {
        batch = (batch - 1).coerceAtLeast(0)
        if (batch == 0) view.publishIme(true)
        return batch > 0
    }

    override fun performEditorAction(editorAction: Int): Boolean {
        if (!multiline) view.pressEnter()
        return true
    }

    override fun performContextMenuAction(id: Int): Boolean {
        when (id) {
            android.R.id.selectAll -> edit(IME_SELECT_ALL)
            android.R.id.copy -> copySelection()
            android.R.id.cut -> {
                copySelection()
                edit(IME_COMMIT, "", 1)
            }
            android.R.id.paste, android.R.id.pasteAsPlainText -> paste()
            else -> return false
        }
        return true
    }

    private fun copySelection() {
        val selected = view.imeText(2, 0)
        if (selected.isNotEmpty()) clipboard?.setPrimaryClip(ClipData.newPlainText(null, selected))
    }

    private fun paste() {
        val text = clipboard?.primaryClip?.getItemAt(0)?.coerceToText(view.context)?.toString().orEmpty()
        if (text.isNotEmpty()) edit(IME_COMMIT, text, 1)
    }

    override fun sendKeyEvent(event: KeyEvent): Boolean {
        val down = event.action == KeyEvent.ACTION_DOWN
        when (event.keyCode) {
            KeyEvent.KEYCODE_DEL -> {
                if (down) edit(IME_BACKSPACE)
                return true
            }
            KeyEvent.KEYCODE_ENTER -> {
                if (down) {
                    if (multiline) edit(IME_COMMIT, "\n", 1) else view.pressEnter()
                }
                return true
            }
        }
        val unicode = event.unicodeChar
        if (unicode > 0 && !event.isCtrlPressed && !event.isAltPressed) {
            if (down) edit(IME_COMMIT, String(Character.toChars(unicode)), 1)
            return true
        }
        return view.dispatchKeyEvent(event)
    }

    override fun requestCursorUpdates(cursorUpdateMode: Int): Boolean {
        view.imeCursorUpdates = cursorUpdateMode and InputConnection.CURSOR_UPDATE_MONITOR != 0
        if (cursorUpdateMode and InputConnection.CURSOR_UPDATE_IMMEDIATE != 0) view.publishIme(true)
        return true
    }

    override fun closeConnection() {
        view.imeCursorUpdates = false
        super.closeConnection()
    }
}
