package com.fluxa.app.data.repository

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DataResultTest {

    @Test
    fun successReturnsValueAndHasNoFailure() {
        val result: DataResult<List<String>> = DataResult.Success(listOf("ok"))

        assertEquals(listOf("ok"), result.getOrDefault(emptyList()))
        assertNull(result.asFailure())
    }

    @Test
    fun networkErrorFallsBackAndReportsOperation() {
        val result: DataResult<List<String>> = DataResult.NetworkError(
            operation = "trakt.watchlist",
            cause = IllegalStateException("boom")
        )

        assertEquals(emptyList<String>(), result.getOrDefault(emptyList()))
        assertEquals("trakt.watchlist", result.asFailure()?.operation)
        assertEquals("boom", result.asFailure()?.message)
    }

    @Test
    fun authUnavailableUsesStableFailureMessage() {
        val result: DataResult<Unit> = DataResult.AuthUnavailable("trakt.collection")

        assertEquals("auth_unavailable", result.asFailure()?.message)
    }
}
