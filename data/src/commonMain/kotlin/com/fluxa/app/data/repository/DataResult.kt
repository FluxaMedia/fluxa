package com.fluxa.app.data.repository

sealed interface DataResult<out T> {
    data class Success<T>(val value: T) : DataResult<T>
    data class NetworkError(val operation: String, val cause: Throwable) : DataResult<Nothing>
    data class ParseError(val operation: String, val cause: Throwable) : DataResult<Nothing>
    data class AuthUnavailable(val operation: String) : DataResult<Nothing>
    data class Unsupported(val operation: String, val reason: String) : DataResult<Nothing>
}

fun <T> DataResult<T>.getOrDefault(defaultValue: T): T = when (this) {
    is DataResult.Success -> value
    else -> defaultValue
}
