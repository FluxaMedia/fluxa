package com.fluxa.app.ui.rust

import android.view.Surface

internal object Mpv {
    init {
        System.loadLibrary("fluxa_mpv")
    }

    @JvmStatic external fun create(): Long
    @JvmStatic external fun initialize(handle: Long): Int
    @JvmStatic external fun destroy(handle: Long)
    @JvmStatic external fun setOption(handle: Long, key: String, value: String): Int
    @JvmStatic external fun setProperty(handle: Long, key: String, value: String): Int
    @JvmStatic external fun getProperty(handle: Long, key: String): String?
    @JvmStatic external fun command(handle: Long, args: Array<String>): Int
    @JvmStatic external fun drain(handle: Long): Boolean
    @JvmStatic external fun attach(handle: Long, surface: Surface): Long
    @JvmStatic external fun detach(handle: Long, ref: Long)
}
