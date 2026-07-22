package com.fluxa.app.data.repository

enum class ExternalSyncProvider { SIMKL, MAL }

enum class ExternalSyncAction { STAMP_SUCCESS, CLEAR_CREDENTIALS, REFRESH_CREDENTIALS, KEEP_CREDENTIALS }

data class MalListUpdate(val malId: Int, val watchedEpisodes: Int?, val status: String)
