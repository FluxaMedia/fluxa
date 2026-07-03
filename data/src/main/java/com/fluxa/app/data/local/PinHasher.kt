package com.fluxa.app.data.local

import java.security.MessageDigest

object PinHasher {
    fun hash(pin: String): String {
        val bytes = MessageDigest.getInstance("SHA-256").digest(pin.toByteArray())
        return bytes.joinToString("") { "%02x".format(it) }
    }
}
