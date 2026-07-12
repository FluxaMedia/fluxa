package com.fluxa.app.common

import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

actual fun epochMillisNow(): Long = System.currentTimeMillis()

internal actual class SimpleLock actual constructor() {
    private val lock = ReentrantLock()
    actual fun <T> withLock(block: () -> T): T = lock.withLock(block)
}
