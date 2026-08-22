package com.fluxa.app.core

import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

inline fun <reified T> Gson.fromState(value: Any?): T? =
    value?.let { runCatching { fromJson(toJsonTree(it), T::class.java) }.getOrNull() }

inline fun <reified T> Gson.fromStateList(value: Any?): List<T> {
    val type = object : TypeToken<List<T>>() {}.type
    return value?.let { runCatching { fromJson<List<T>>(toJsonTree(it), type) }.getOrNull() } ?: emptyList()
}
