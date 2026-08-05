package com.fluxa.app.common

expect object PlatformLog {
    fun d(tag: String, message: String)
    fun w(tag: String, message: String, throwable: Throwable? = null)
}
