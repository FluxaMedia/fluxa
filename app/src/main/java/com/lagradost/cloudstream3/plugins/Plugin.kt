package com.lagradost.cloudstream3.plugins

import android.content.Context
import android.content.res.Resources

abstract class Plugin : BasePlugin() {
    var resources: Resources? = null

    val name: String
        get() = this::class.java.simpleName

    open fun load(context: Context) {
        load()
    }
}
