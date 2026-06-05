package com.fluxa.app.data.remote

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import retrofit2.http.GET
import retrofit2.http.Path

class StremioServiceTest {

    @Test
    fun catalogGenreEndpointBindsGenreAsPathExtra() {
        val method = StremioService::class.java.methods.single { it.name == "getCatalogWithGenre" }

        assertEquals("catalog/{type}/{id}/genre={genre}.json", method.getAnnotation(GET::class.java)?.value)

        val parameterNames = method.parameterAnnotations
            .flatMap { annotations -> annotations.filterIsInstance<Path>() }
            .map { it.value }

        assertTrue(parameterNames.containsAll(listOf("type", "id", "genre")))
    }
}
