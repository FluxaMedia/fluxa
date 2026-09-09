package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

private val homeCategoryGson = Gson()
private val homeCategoryListType = object : TypeToken<List<HomeCategory>>() {}.type

fun filterHomeCategoriesWithCore(
    categories: List<HomeCategory>,
    filter: String,
): List<HomeCategory> = homeCategoryGson.fromJson<List<HomeCategory>>(
    FluxaCoreNative.filterHomeCategoriesJson(homeCategoryGson.toJson(categories), filter),
    homeCategoryListType,
) ?: emptyList()
