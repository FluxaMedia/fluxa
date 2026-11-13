package com.fluxa.app.ui.rust

import android.content.ClipboardManager
import android.content.Context
import android.graphics.Rect
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.view.View

internal class TextToolbar(private val view: FluxaNativeRendererView) {
    private var mode: ActionMode? = null
    private var closing = false
    private val anchor = Rect()
    private var selection = false
    private var secret = false

    private val clipboard
        get() = view.context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager

    private fun hasPaste() = clipboard?.primaryClipDescription?.hasMimeType("text/*") == true

    private fun populate(menu: Menu) {
        val context = view.context
        val text = view.imeText(3, 0)
        val copyable = selection && !secret
        if (copyable) {
            menu.add(Menu.NONE, android.R.id.cut, 0, context.getString(android.R.string.cut))
            menu.add(Menu.NONE, android.R.id.copy, 1, context.getString(android.R.string.copy))
        }
        if (hasPaste()) {
            menu.add(Menu.NONE, android.R.id.paste, 2, context.getString(android.R.string.paste))
        }
        if (text.isNotEmpty() && view.imeText(2, 0).length < text.length) {
            menu.add(Menu.NONE, android.R.id.selectAll, 3, context.getString(android.R.string.selectAll))
        }
    }

    private val callback = object : ActionMode.Callback2() {
        override fun onCreateActionMode(mode: ActionMode, menu: Menu): Boolean {
            populate(menu)
            return menu.size() > 0
        }

        override fun onPrepareActionMode(mode: ActionMode, menu: Menu): Boolean {
            menu.clear()
            populate(menu)
            return menu.size() > 0
        }

        override fun onActionItemClicked(mode: ActionMode, item: MenuItem): Boolean {
            view.performTextAction(item.itemId)
            if (item.itemId == android.R.id.selectAll) {
                mode.invalidate()
            } else {
                view.dismissTextToolbar()
                close()
            }
            return true
        }

        override fun onDestroyActionMode(mode: ActionMode) {
            if (this@TextToolbar.mode === mode) this@TextToolbar.mode = null
            if (!closing) view.dismissTextToolbar()
        }

        override fun onGetContentRect(mode: ActionMode, view: View, outRect: Rect) {
            outRect.set(anchor)
        }
    }

    fun sync(spec: FloatArray?) {
        if (spec == null) {
            close()
            return
        }
        val density = view.resources.displayMetrics.density
        val next = Rect(
            (spec[0] * density).toInt(),
            (spec[1] * density).toInt(),
            (spec[2] * density).toInt(),
            (spec[3] * density).toInt(),
        )
        val hasSelection = spec[4] > 0f
        val isSecret = spec[5] > 0f
        val current = mode
        val changed = hasSelection != selection || isSecret != secret
        selection = hasSelection
        secret = isSecret
        if (current == null) {
            anchor.set(next)
            mode = view.startActionMode(callback, ActionMode.TYPE_FLOATING)
            return
        }
        if (changed) current.invalidate()
        if (next != anchor) {
            anchor.set(next)
            current.invalidateContentRect()
        }
    }

    fun close() {
        val current = mode ?: return
        mode = null
        closing = true
        current.finish()
        closing = false
    }
}
