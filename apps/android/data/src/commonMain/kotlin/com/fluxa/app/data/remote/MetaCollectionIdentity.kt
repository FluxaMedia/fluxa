package com.fluxa.app.data.remote

val Meta.typeAndIdKey: String
    get() = "$type:$id"

fun Iterable<Meta>.distinctByTypeAndId(): List<Meta> = distinctBy(Meta::typeAndIdKey)
