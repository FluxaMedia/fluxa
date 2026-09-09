package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta

const val CONTINUE_WATCHING_CATEGORY_ID = "continue_watching"
const val UPCOMING_CATEGORY_ID = "upcoming"

fun HomeCategory.isContinueWatchingCategory(): Boolean = id == CONTINUE_WATCHING_CATEGORY_ID
fun HomeCategory.isUpcomingCategory(): Boolean = id == UPCOMING_CATEGORY_ID
fun HomeCategory.isContinueWatchingOrUpcomingCategory(): Boolean =
    isContinueWatchingCategory() || isUpcomingCategory()
