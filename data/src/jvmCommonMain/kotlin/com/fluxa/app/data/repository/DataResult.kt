@file:JvmName("DataFailureMapping")

package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative

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
