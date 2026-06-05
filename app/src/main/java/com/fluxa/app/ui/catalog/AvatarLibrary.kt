package com.fluxa.app.ui.catalog

data class AvatarCategory(
    val showName: String,
    val imdbId: String,
    val characters: List<AvatarCharacter> = emptyList() // Will be populated dynamically
)

data class AvatarCharacter(
    val name: String,
    val url: String
)

object AvatarLibrary {
    private val defaultCategories = listOf(
        AvatarCategory("The Boys", "TheBoys"),
        AvatarCategory("Breaking Bad", "BreakingBad"),
        AvatarCategory("Peaky Blinders", "PeakyBlinders"),
        AvatarCategory("Invincible", "Invincible"),
        AvatarCategory("Avatar: The Last Airbender", "ATLA")
    )

    val categories: List<AvatarCategory> = defaultCategories
}
