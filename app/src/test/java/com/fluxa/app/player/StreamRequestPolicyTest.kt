package com.fluxa.app.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class StreamRequestPolicyTest {
    @Test
    fun keepsOnlyCleanStreamHeaders() {
        val headers = StreamRequestPolicy.headersFor(
            url = "https://cdn.example/video.m3u8",
            streamHeaders = mapOf("X-Test" to "ok", "" to "ignored", "Blank" to "")
        )

        assertEquals(mapOf("X-Test" to "ok"), headers)
    }

    @Test
    fun doesNotInjectKnownHostHeaders() {
        val headers = StreamRequestPolicy.headersFor(
            url = "https://cdn.vidmoly.me/video.mp4"
        )

        assertTrue(headers.isEmpty())
    }

    @Test
    fun preservesExplicitStreamRefererOnly() {
        val headers = StreamRequestPolicy.headersFor(
            url = "https://stream.tv2.example/video.mp4",
            streamHeaders = mapOf("Referer" to "https://source.example/")
        )

        assertEquals("https://source.example/", headers["Referer"])
        assertFalse(headers.containsKey("Origin"))
    }

    @Test
    fun refererPolicyIsDisabled() {
        assertNull(StreamRequestPolicy.refererFor("https://vidmoly.me/video.mp4"))
    }
}
