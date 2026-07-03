package com.fluxa.app.core.rust.models

data class NativeCalendarNotificationContent(
    val items: List<Map<String, Any?>> = emptyList(),
    val keys: List<String> = emptyList()
)
