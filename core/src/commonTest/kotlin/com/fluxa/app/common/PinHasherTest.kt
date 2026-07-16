package com.fluxa.app.common

import kotlin.test.Test
import kotlin.test.assertEquals

class PinHasherTest {
    @Test
    fun hashesWithPortableSha256() {
        assertEquals("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", PinHasher.hash(""))
        assertEquals("03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4", PinHasher.hash("1234"))
    }
}
