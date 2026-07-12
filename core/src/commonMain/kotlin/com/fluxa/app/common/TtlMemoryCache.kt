package com.fluxa.app.common

class TtlMemoryCache<T>(
    private val maxEntries: Int,
    private val ttlMillis: Long,
    private val nowMillis: () -> Long = ::epochMillisNow
) {
    private data class Entry<T>(
        val value: T,
        val expiresAtMillis: Long,
        val insertionOrder: Long
    )

    private val lock = SimpleLock()
    private val entries = mutableMapOf<String, Entry<T>>()
    private var nextInsertionOrder = 0L

    fun get(key: String): T? = lock.withLock {
        val entry = entries[key] ?: return@withLock null
        if (nowMillis() < entry.expiresAtMillis) entry.value else {
            entries.remove(key); null
        }
    }

    fun put(key: String, value: T) = lock.withLock {
        val now = nowMillis()
        trimExpired(now)
        entries[key] = Entry(
            value = value,
            expiresAtMillis = now + ttlMillis,
            insertionOrder = nextInsertionOrder++
        )
        trimToMaxSize()
    }

    fun firstWithPrefix(prefix: String): T? = lock.withLock {
        val now = nowMillis()
        trimExpired(now)
        val key = entries.keys.asSequence().filter { it.startsWith(prefix) }.minOrNull() ?: return@withLock null
        entries[key]?.value
    }

    fun invalidatePrefix(prefix: String) = lock.withLock {
        entries.keys.removeAll { it.startsWith(prefix) }
    }

    fun size(): Int = lock.withLock {
        trimExpired(nowMillis())
        entries.size
    }

    private fun trimExpired(now: Long) {
        entries.entries.removeAll { (_, entry) -> now >= entry.expiresAtMillis }
    }

    private fun trimToMaxSize() {
        val excess = entries.size - maxEntries
        if (excess <= 0) return
        entries.entries
            .sortedBy { it.value.insertionOrder }
            .take(excess)
            .forEach { entries.remove(it.key) }
    }
}
