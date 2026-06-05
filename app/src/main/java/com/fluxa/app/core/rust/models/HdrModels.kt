package com.fluxa.app.core.rust.models

import com.google.gson.annotations.SerializedName

data class NativeDvProxyPlan(
    val action: String = "none",
    @SerializedName("rpuMode") val rpuMode: Int = 2,
    val reason: String = "",
    val profile: String = "unknown",
    val compatibility: String = "none",
    val safety: String = "none",
    val limitations: List<String> = emptyList()
)

data class NativeDolbyVisionRpuInfo(
    val ok: Boolean = false,
    val profile: Int? = null,
    @SerializedName("el_type")
    val elType: String? = null,
    val error: String? = null
)

data class NativeDolbyVisionRpuConvertResult(
    val ok: Boolean = false,
    @SerializedName("profile_before")
    val profileBefore: Int? = null,
    @SerializedName("profile_after")
    val profileAfter: Int? = null,
    @SerializedName("el_type_before")
    val elTypeBefore: String? = null,
    @SerializedName("el_type_after")
    val elTypeAfter: String? = null,
    @SerializedName("rpu_hex")
    val rpuHex: String? = null,
    @SerializedName("rpu_base64")
    val rpuBase64: String? = null,
    val error: String? = null
)
