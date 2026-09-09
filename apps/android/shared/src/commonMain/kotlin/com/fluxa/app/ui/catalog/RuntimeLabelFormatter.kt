package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative

fun formatRuntimeLabel(raw: String?): String? = FluxaCoreNative.formatRuntimeLabel(raw)
