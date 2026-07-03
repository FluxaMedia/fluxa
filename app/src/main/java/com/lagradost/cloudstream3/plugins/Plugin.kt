package com.lagradost.cloudstream3.plugins

import android.content.Context
import android.content.res.Resources
import android.util.Log
import com.lagradost.cloudstream3.actions.VideoClickAction
import com.lagradost.cloudstream3.actions.VideoClickActionHolder

abstract class Plugin : BasePlugin() {
    var resources: Resources? = null

    var openSettings: ((context: Context) -> Unit)? = null

    val name: String
        get() = this::class.java.simpleName

    open fun load(context: Context) {
        load()
    }

    fun registerVideoClickAction(element: VideoClickAction) {
        Log.i(PLUGIN_TAG, "Adding ${element.name} VideoClickAction")
        element.sourcePlugin = this.filename
        VideoClickActionHolder.allVideoClickActions.add(element)
    }
}
