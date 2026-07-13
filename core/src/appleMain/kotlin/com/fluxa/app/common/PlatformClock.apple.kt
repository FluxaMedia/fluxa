package com.fluxa.app.common

import platform.Foundation.NSDate
import platform.Foundation.NSLock
import platform.Foundation.timeIntervalSince1970

actual fun epochMillisNow(): Long = (NSDate().timeIntervalSince1970 * 1000).toLong()

internal actual class SimpleLock actual constructor() {
    private val lock = NSLock()

    actual fun <T> withLock(block: () -> T): T {
        lock.lock()
        try {
            return block()
        } finally {
            lock.unlock()
        }
    }
}
