package com.fluxa.app.core.rust.models

data class NativeTraktCommentsRequest(
    val resource: String,
    val id: String,
    val lookupType: String,
    val wantType: String
)
