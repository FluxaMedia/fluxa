package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative

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

fun DataResult<*>.asFailure(): DataFailure? = when (this) {
    is DataResult.Success -> null
    is DataResult.AuthUnavailable -> rustFailure(operation, "AuthUnavailable")
    is DataResult.NetworkError -> rustFailure(
        operation = operation,
        kind = "NetworkError",
        message = cause.message,
        throwableClass = cause::class.java.simpleName
    )
    is DataResult.ParseError -> rustFailure(
        operation = operation,
        kind = "ParseError",
        message = cause.message,
        throwableClass = cause::class.java.simpleName
    )
    is DataResult.Unsupported -> rustFailure(operation, "Unsupported", reason = reason)
}

private fun rustFailure(
    operation: String,
    kind: String,
    message: String? = null,
    throwableClass: String? = null,
    reason: String? = null
): DataFailure {
    val policy = FluxaCoreNative.dataFailurePolicy(
        operation = operation,
        kind = kind,
        message = message,
        throwableClass = throwableClass,
        reason = reason
    )
    return DataFailure(operation = policy.operation.ifBlank { operation }, message = policy.message)
}
