package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.local.WatchlistManager.WatchlistMembershipEntry
import com.google.gson.JsonArray
import com.google.gson.JsonObject

object ExternalSyncMergeBridge {
    data class MembershipMergePlan(
        val applyLocalAdd: List<String>,
        val pushRemoteAdd: List<String>,
        val pushRemoteRemove: List<String>
    )

    data class RemoteMembershipItem(val id: String, val updatedAt: Long)

    fun mergeWatchlist(local: List<WatchlistMembershipEntry>, remote: List<RemoteMembershipItem>): MembershipMergePlan =
        merge("mergeWatchlistTimestamped", local, remote)

    fun mergeWatched(local: List<WatchlistMembershipEntry>, remote: List<RemoteMembershipItem>): MembershipMergePlan =
        merge("mergeWatchedTimestamped", local, remote)

    private fun merge(
        method: String,
        local: List<WatchlistMembershipEntry>,
        remote: List<RemoteMembershipItem>
    ): MembershipMergePlan {
        val localJson = JsonArray().apply {
            local.forEach { entry ->
                add(JsonObject().apply {
                    addProperty("id", entry.id)
                    addProperty("active", entry.active)
                    addProperty("updatedAt", entry.updatedAt)
                })
            }
        }
        val remoteJson = JsonArray().apply {
            remote.forEach { entry ->
                add(JsonObject().apply {
                    addProperty("id", entry.id)
                    addProperty("updatedAt", entry.updatedAt)
                })
            }
        }
        val args = JsonObject().apply {
            add("local", localJson)
            add("remote", remoteJson)
        }
        val result = FluxaCoreUniFfi.coreInvokeValue(method, args.toString()).asJsonObject
        val apply = result.getAsJsonObject("toApplyLocal")
        val push = result.getAsJsonObject("toPushRemote")
        return MembershipMergePlan(
            applyLocalAdd = apply.getAsJsonArray("add").map { it.asString },
            pushRemoteAdd = push.getAsJsonArray("add").map { it.asString },
            pushRemoteRemove = push.getAsJsonArray("remove").map { it.asString }
        )
    }
}
