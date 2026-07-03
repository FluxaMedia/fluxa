package com.fluxa.app.data.repository

import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject
import javax.inject.Singleton

data class DataFailure(
    val operation: String,
    val message: String,
    val occurredAtMillis: Long = System.currentTimeMillis()
)

@Singleton
class DataFailureReporter @Inject constructor() {
    private val _latestFailure = MutableStateFlow<DataFailure?>(null)
    val latestFailure: StateFlow<DataFailure?> = _latestFailure.asStateFlow()

    fun report(operation: String, throwable: Throwable) {
        val failure = DataFailure(
            operation = operation,
            message = throwable.message ?: throwable::class.java.simpleName
        )
        _latestFailure.value = failure
        Log.w("DataFailure", "$operation failed: ${failure.message}", throwable)
    }

    fun report(failure: DataFailure) {
        _latestFailure.value = failure
        Log.w("DataFailure", "${failure.operation} failed: ${failure.message}")
    }
}
