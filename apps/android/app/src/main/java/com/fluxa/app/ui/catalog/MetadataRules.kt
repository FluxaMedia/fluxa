package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.rust.FluxaCoreNative

internal fun detailIsUpcoming(date: String?): Boolean =
    date != null && FluxaCoreNative.releaseDateUpcoming(date, ReleaseDateUtils.todayIso())
