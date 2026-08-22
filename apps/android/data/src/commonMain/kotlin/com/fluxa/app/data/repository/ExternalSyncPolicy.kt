package com.fluxa.app.data.repository

enum class ExternalSyncProvider { SIMKL }

enum class ExternalSyncAction { STAMP_SUCCESS, CLEAR_CREDENTIALS, REFRESH_CREDENTIALS, KEEP_CREDENTIALS }
