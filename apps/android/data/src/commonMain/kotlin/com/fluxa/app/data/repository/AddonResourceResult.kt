package com.fluxa.app.data.repository

sealed class AddonResourceResult<out T> {
    data class Success<T>(val value: T, val url: String) : AddonResourceResult<T>()
    data class Empty(val url: String) : AddonResourceResult<Nothing>()
    data class NetworkError(val url: String, val cause: Throwable? = null, val statusCode: Int? = null) : AddonResourceResult<Nothing>()
    data class ParseError(val url: String, val cause: Throwable) : AddonResourceResult<Nothing>()
    data class AddonUnsupported(val addonName: String, val resource: String, val type: String, val id: String) : AddonResourceResult<Nothing>()
}
