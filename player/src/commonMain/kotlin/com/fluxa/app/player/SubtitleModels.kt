package com.fluxa.app.player

data class ExternalSubtitleTrack(
    val url: String,
    val language: String?,
    val label: String?
)

data class NativeAssTrack(
    val id: String,
    val label: String?,
    val language: String?,
    val assData: ByteArray,
    val fonts: List<NativeAssFont> = emptyList()
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is NativeAssTrack) return false
        return id == other.id && label == other.label && language == other.language &&
            assData.contentEquals(other.assData) && fonts == other.fonts
    }

    override fun hashCode(): Int {
        var result = id.hashCode()
        result = 31 * result + (label?.hashCode() ?: 0)
        result = 31 * result + (language?.hashCode() ?: 0)
        result = 31 * result + assData.contentHashCode()
        result = 31 * result + fonts.hashCode()
        return result
    }
}

data class NativeAssFont(
    val name: String,
    val data: ByteArray
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is NativeAssFont) return false
        return name == other.name && data.contentEquals(other.data)
    }

    override fun hashCode(): Int {
        var result = name.hashCode()
        result = 31 * result + data.contentHashCode()
        return result
    }
}
