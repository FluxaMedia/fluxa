package com.fluxa.app.common

actual object PlatformLog {
    actual fun d(tag: String, message: String) {
        android.util.Log.d(tag, message)
    }

    actual fun w(tag: String, message: String, throwable: Throwable?) {
        android.util.Log.w(tag, message, throwable)
    }
}
