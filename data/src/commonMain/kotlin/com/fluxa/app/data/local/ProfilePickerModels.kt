package com.fluxa.app.data.local

data class AvatarRecord(
    val name: String = "",
    val url: String = ""
)

data class AvatarPackRecord(
    val id: String,
    val repositoryUrl: String,
    val title: String,
    val avatars: List<AvatarRecord> = emptyList()
)

data class ProfilePickerSettingsRecord(
    val backgroundUrl: String? = null,
    val avatarPacks: List<AvatarPackRecord> = emptyList()
)
