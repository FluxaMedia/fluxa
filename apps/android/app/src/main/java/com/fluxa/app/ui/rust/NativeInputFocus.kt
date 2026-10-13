package com.fluxa.app.ui.rust

object NativeInputFocus {
    @Volatile
    var listener: Runnable? = null

    @JvmStatic
    fun changed() {
        listener?.run()
    }
}
