package com.fluxa.app.common

actual object PlatformLog {
    actual fun d(tag: String, message: String) {
        println("D/$tag: $message")
    }

    actual fun w(tag: String, message: String, throwable: Throwable?) {
        println("W/$tag: $message${throwable?.let { " - $it" } ?: ""}")
    }
}
