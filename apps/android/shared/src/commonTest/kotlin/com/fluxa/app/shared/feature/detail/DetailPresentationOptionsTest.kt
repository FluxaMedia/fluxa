package com.fluxa.app.shared.feature.detail

import kotlin.test.Test
import kotlin.test.assertEquals

class DetailPresentationOptionsTest {
    @Test
    fun detailScreenStyleParsingIsStableAndBackwardsCompatible() {
        assertEquals(DetailScreenStyle.Cinematic, DetailScreenStyle.from(null))
        assertEquals(DetailScreenStyle.Cinematic, DetailScreenStyle.from("unknown"))
        assertEquals(DetailScreenStyle.Cinematic, DetailScreenStyle.from("CINEMATIC"))
        assertEquals(DetailScreenStyle.Classic, DetailScreenStyle.from("classic"))
        assertEquals(DetailScreenStyle.Compact, DetailScreenStyle.from(" compact "))
    }

    @Test
    fun cinematicIsTheDefaultPresentation() {
        val options = DetailPresentationOptions()
        assertEquals(DetailScreenStyle.Cinematic, options.screenStyle)
        assertEquals("carousel", options.episodeCardsLayout)
    }
}
