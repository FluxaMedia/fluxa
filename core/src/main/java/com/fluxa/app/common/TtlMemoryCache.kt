package com.fluxa.app.common

import java.util.concurrent.ConcurrentHashMap

class TtlMemoryCache<T>(
    private val maxEntries: Int,
    private val ttlMillis: Long,
    private val nowMillis: () -> Long = System::currentTimeMillis
) {
    private data class Entry<T>(
        val value: T,
        val expiresAtMillis: Long,
        val insertionOrder: Long
    )

    private val entries = ConcurrentHashMap<String, Entry<T>>()
    private var nextInsertionOrder = 0L

    fun get(key: String): T? {
        val entry = entries[key] ?: return null
        return if (nowMillis() < entry.expiresAtMillis) entry.value else {
            entries.remove(key); null
        }
    }

    @Synchronized
    fun put(key: String, value: T) {
        val now = nowMillis()
        trimExpired(now)
        entries[key] = Entry(
            value = value,
            expiresAtMillis = now + ttlMillis,
            insertionOrder = nextInsertionOrder++
        )
        trimToMaxSize()
    }

    @Synchronized
    fun firstWithPrefix(prefix: String): T? {
        val now = nowMillis()
        trimExpired(now)
        val key = entries.keys.asSequence().filter { it.startsWith(prefix) }.minOrNull() ?: return null
        return entries[key]?.value
    }

    @Synchronized
    fun invalidatePrefix(prefix: String) {
        entries.keys.removeAll { it.startsWith(prefix) }
    }

    @Synchronized
    fun size(): Int {
        trimExpired(nowMillis())
        return entries.size
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
