package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformHttpClient
import com.fluxa.app.data.platform.PlatformHttpRequest
import com.fluxa.app.data.platform.PlatformHttpResponse
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class PlatformResourceClientTest {
    @Test
    fun successfulResponseIsParsed() = runTest {
        val client = PlatformResourceClient(FakeHttpClient(PlatformHttpResponse(200, body = "value")))

        val result = client.get("https://example.test") { it.uppercase() }

        assertEquals("VALUE", assertIs<AddonResourceResult.Success<String>>(result).value)
    }

    @Test
    fun unsuccessfulResponsePreservesStatusCode() = runTest {
        val client = PlatformResourceClient(FakeHttpClient(PlatformHttpResponse(503)))

        val result = client.get("https://example.test") { it }

        assertEquals(503, assertIs<AddonResourceResult.NetworkError>(result).statusCode)
    }

    private class FakeHttpClient(
        private val response: PlatformHttpResponse
    ) : PlatformHttpClient {
        override suspend fun execute(request: PlatformHttpRequest): PlatformHttpResponse = response
    }
}
