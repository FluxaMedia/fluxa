package com.fluxa.app.shared.feature.profile

import java.security.MessageDigest
import java.security.SecureRandom
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec

interface ProfileBase64Codec {
    fun encode(bytes: ByteArray): String
    fun decode(value: String): ByteArray?
}

/** Shared PBKDF2 format/verification; platforms only provide Base64 encoding. */
class JvmPbkdf2PinHasher(
    private val base64: ProfileBase64Codec,
) {
    fun hash(pin: String): String {
        val salt = ByteArray(SALT_BYTES).also(SecureRandom()::nextBytes)
        val derived = derive(pin, salt, ITERATIONS)
        return "$ALGORITHM_PREFIX:$ITERATIONS:${base64.encode(salt)}:${base64.encode(derived)}"
    }

    fun verify(pin: String, storedHash: String): Boolean {
        val parts = storedHash.split(':')
        if (parts.size != 4 || parts[0] != ALGORITHM_PREFIX) return false
        val iterations = parts[1].toIntOrNull()?.takeIf { it > 0 } ?: return false
        val salt = base64.decode(parts[2]) ?: return false
        val expected = base64.decode(parts[3]) ?: return false
        return MessageDigest.isEqual(derive(pin, salt, iterations), expected)
    }

    private fun derive(pin: String, salt: ByteArray, iterations: Int): ByteArray =
        SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256")
            .generateSecret(PBEKeySpec(pin.toCharArray(), salt, iterations, KEY_LENGTH_BITS))
            .encoded

    private companion object {
        const val ALGORITHM_PREFIX = "pbkdf2-sha256"
        const val ITERATIONS = 210_000
        const val KEY_LENGTH_BITS = 256
        const val SALT_BYTES = 16
    }
}
