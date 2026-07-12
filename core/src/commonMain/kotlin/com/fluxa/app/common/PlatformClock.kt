package com.fluxa.app.common

expect fun epochMillisNow(): Long

internal expect class SimpleLock() {
    fun <T> withLock(block: () -> T): T
}
