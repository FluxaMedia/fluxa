package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.util.Locale

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun AddonManifestSummary(
    addon: CommunityAddon,
    lang: String,
    isEnabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    val manifest = addon.manifest ?: return
    val chips = buildList {
        if (isEnabled) add(AppStrings.t(lang, "addons.active"))
        val resources = manifest.resourceSpecs().map { it.name }.distinct().size
        if (resources > 0) add(AppStrings.format(lang, "addons.resources_count", resources))
        val catalogs = manifest.catalogs.orEmpty().size
        if (catalogs > 0) add(AppStrings.format(lang, "addons.catalogs_count", catalogs))
        if (addon.configurable) add(AppStrings.t(lang, "addons.configurable"))
    }
    if (chips.isEmpty()) return
    FlowRow(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        chips.forEach { chip -> AddonInfoChip(chip) }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun AddonStoreCounters(
    installedAddons: List<CommunityAddon>,
    lang: String,
    modifier: Modifier = Modifier
) {
    val total = installedAddons.size
    val active = installedAddons.count { addon ->
        addon.manifest?.resources.orEmpty().isNotEmpty()
    }
    val catalogs = installedAddons.sumOf { it.manifest?.catalogs.orEmpty().size }

    FlowRow(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        AddonCounterChip(AppStrings.format(lang, "addons.count_existing", total))
        AddonCounterChip(AppStrings.format(lang, "addons.count_active", active))
        AddonCounterChip(AppStrings.format(lang, "addons.count_catalogs", catalogs))
    }
}

@Composable
private fun AddonCounterChip(label: String) {
    Text(
        text = label,
        color = Color.White,
        fontSize = 12.sp,
        fontWeight = FontWeight.Bold,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier
            .background(Color.White.copy(alpha = 0.10f), RoundedCornerShape(999.dp))
            .padding(horizontal = 12.dp, vertical = 8.dp)
    )
}

@Composable
private fun AddonInfoChip(
    label: String,
    modifier: Modifier = Modifier,
    strong: Boolean = false
) {
    Text(
        text = label,
        color = Color.White.copy(alpha = if (strong) 0.88f else 0.72f),
        fontSize = 11.sp,
        lineHeight = 15.sp,
        fontWeight = if (strong) FontWeight.Bold else FontWeight.Medium,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
            .background(Color.White.copy(alpha = if (strong) 0.10f else 0.06f), RoundedCornerShape(999.dp))
            .padding(horizontal = 10.dp, vertical = 6.dp)
    )
}

@Composable
internal fun AddonVersionBadge(
    version: String?,
    lang: String,
    modifier: Modifier = Modifier
) {
    val label = version?.toAddonVersionLabel(lang) ?: return
    Text(
        text = label,
        color = Color.White.copy(alpha = 0.46f),
        fontSize = 11.sp,
        lineHeight = 14.sp,
        fontWeight = FontWeight.Medium,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
    )
}

private fun String.toAddonVersionLabel(lang: String): String? {
    val normalized = trim().removePrefix("v").removePrefix("V").takeIf { it.isNotBlank() } ?: return null
    return AppStrings.format(lang, "addons.version", normalized)
}

internal fun addonManifestSummaryChips(manifest: AddonManifest, lang: String): List<String> {
    val resources = manifest.resourceSpecs().map { it.name }.distinct().size
    val catalogs = manifest.catalogs.orEmpty().size
    return buildList {
        add(AppStrings.format(lang, "addons.resources_count", resources))
        add(AppStrings.format(lang, "addons.catalogs_count", catalogs))
    }
}

internal fun addonManifestCapabilityRows(manifest: AddonManifest, lang: String): List<String> {
    val rows = manifest.resourceSpecs().groupBy { spec ->
        spec.effectiveTypes(manifest).ifEmpty {
            manifest.catalogs.orEmpty().mapNotNull { it.type }
        }.mapNotNull { it.toAddonTypeLabel(lang) }.distinct()
    }

    return rows.map { (types, specs) ->
        val typeText = types.takeIf { it.isNotEmpty() }?.joinToString(" / ")
            ?: AppStrings.t(lang, "auto.all")
        val resources = specs.map { it.name.toCanonicalAddonResource() }.distinct().joinToString(", ")
        val idRules = specs.flatMap { it.effectiveIdPrefixes(manifest) }.distinct().size
        "$typeText - $resources - ${AppStrings.format(lang, "addons.id_rules_count", idRules)}"
    }
}

private data class AddonResourceSpec(
    val name: String,
    val types: List<String>?,
    val idPrefixes: List<String>?
) {
    fun effectiveTypes(manifest: AddonManifest): List<String> {
        return types ?: manifest.types.orEmpty()
    }

    fun effectiveIdPrefixes(manifest: AddonManifest): List<String> {
        if (name.toCanonicalAddonResource() == "catalog") return emptyList()
        return idPrefixes ?: manifest.idPrefixes.orEmpty()
    }
}

private fun AddonManifest.resourceSpecs(): List<AddonResourceSpec> {
    return resources.orEmpty().mapNotNull { resource ->
        when (resource) {
            is String -> AddonResourceSpec(resource, null, null)
            is Map<*, *> -> {
                val name = resource["name"]?.toString()?.takeIf { it.isNotBlank() } ?: return@mapNotNull null
                AddonResourceSpec(
                    name = name,
                    types = (resource["types"] ?: resource["type"]).toStringList(),
                    idPrefixes = (resource["idPrefixes"] ?: resource["idPrefix"]).toStringList()
                )
            }
            else -> null
        }
    }
}

private fun Any?.toStringList(): List<String>? {
    val values = when (this) {
        is Iterable<*> -> mapNotNull { it?.toString()?.takeIf(String::isNotBlank) }
        is Array<*> -> mapNotNull { it?.toString()?.takeIf(String::isNotBlank) }
        is String -> listOf(this).filter { it.isNotBlank() }
        else -> emptyList()
    }
    return values.takeIf { it.isNotEmpty() }
}

private fun String.toCanonicalAddonResource(): String {
    return lowercase(Locale.ROOT)
        .removeSuffix("s")
        .let { if (it == "metadata") "meta" else it }
}

private fun String.toAddonTypeLabel(lang: String): String? {
    return when (lowercase(Locale.ROOT)) {
        "movie", "movies" -> AppStrings.t(lang, "auto.movies")
        "series", "show", "shows", "tv" -> AppStrings.t(lang, "auto.tv_shows")
        "channel" -> AppStrings.t(lang, "addons.type_channels")
        else -> replaceFirstChar { if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString() }
    }
}
