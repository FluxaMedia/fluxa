package com.fluxa.app.core.rust.models

import kotlinx.serialization.SerialName

data class NativeDvProxyPlan(
    val action: String = "none",
    @SerialName("rpuMode") val rpuMode: Int = 2,
    val reason: String = "",
    val profile: String = "unknown",
    val compatibility: String = "none",
    val safety: String = "none",
    val limitations: List<String> = emptyList()
)

data class NativeDolbyVisionRpuInfo(
    val ok: Boolean = false,
    val profile: Int? = null,
    @SerialName("el_type") val elType: String? = null,
    val error: String? = null
)

data class NativeDolbyVisionRpuConvertResult(
    val ok: Boolean = false,
    @SerialName("profile_before") val profileBefore: Int? = null,
    @SerialName("profile_after") val profileAfter: Int? = null,
    @SerialName("el_type_before") val elTypeBefore: String? = null,
    @SerialName("el_type_after") val elTypeAfter: String? = null,
    @SerialName("rpu_hex") val rpuHex: String? = null,
    @SerialName("rpu_base64") val rpuBase64: String? = null,
    val error: String? = null
)
