package com.anandacahya.kusuri.ui.devices

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import com.anandacahya.kusuri.data.model.EnrichedDeviceSpecsResponse

@Composable
fun DeviceDetailDialog(
    specs: EnrichedDeviceSpecsResponse,
    onDismiss: () -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val scrollState = rememberScrollState()

    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                val imageUrl = specs.catalog_specs?.image_url ?: specs.catalog_specs?.picture_url
                if (!imageUrl.isNullOrBlank()) {
                    AsyncImage(
                        model = imageUrl,
                        contentDescription = specs.marketing_name ?: specs.name,
                        modifier = Modifier
                            .size(54.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant),
                        contentScale = ContentScale.Fit
                    )
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = specs.marketing_name ?: specs.name ?: "Device Details",
                        style = MaterialTheme.typography.titleLarge
                    )
                    Text(
                        text = "${specs.device_type?.uppercase() ?: "DEVICE"} • Model: ${specs.raw_model ?: "N/A"}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            }
        },
        text = {
            Column(modifier = Modifier.fillMaxWidth()) {
                TabRow(selectedTabIndex = selectedTab) {
                    Tab(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        text = { Text("Hardware Specs") }
                    )
                    Tab(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        text = { Text("Live Telemetry") }
                    )
                }

                Spacer(modifier = Modifier.height(12.dp))

                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 420.dp)
                        .verticalScroll(scrollState),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    if (selectedTab == 0) {
                        val catalog = specs.catalog_specs
                        if (catalog != null) {
                            val sources = catalog.raw_specs?.sources 
                                ?: if (!catalog.gsmarena_url.isNullOrBlank()) listOf("GSM Arena") else emptyList()
                            if (sources.isNotEmpty()) {
                                Column(modifier = Modifier.padding(bottom = 6.dp)) {
                                    Text(
                                        text = "Sumber Kolektif Terverifikasi:",
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.primary
                                    )
                                    Spacer(modifier = Modifier.height(4.dp))
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                                    ) {
                                        sources.forEach { source ->
                                            Surface(
                                                color = MaterialTheme.colorScheme.primaryContainer,
                                                shape = RoundedCornerShape(12.dp)
                                            ) {
                                                Text(
                                                    text = source,
                                                    style = MaterialTheme.typography.labelSmall,
                                                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                                                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp)
                                                )
                                            }
                                        }
                                    }
                                }
                                HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                            }

                            SpecRow("Brand", catalog.brand ?: catalog.retail_branding ?: specs.retail_branding ?: "-")
                            SpecRow("Chipset / SoC", catalog.chipset ?: specs.chipset ?: "-")
                            val cpu = catalog.cpu ?: "-"
                            val gpu = catalog.gpu
                            SpecRow("CPU & GPU", if (!gpu.isNullOrBlank()) "$cpu ($gpu)" else cpu)
                            val displayType = catalog.display_type ?: ""
                            val displaySize = catalog.display_size ?: ""
                            val displayRes = catalog.display_resolution ?: ""
                            val displayCombined = listOf(displayType, displaySize, displayRes).filter { it.isNotBlank() }.joinToString(" • ")
                            SpecRow("Display", if (displayCombined.isNotBlank()) displayCombined else "-")
                            SpecRow("Main Camera", catalog.main_camera ?: catalog.camera_spec ?: "-")
                            if (!catalog.selfie_camera.isNullOrBlank()) {
                                SpecRow("Selfie Camera", catalog.selfie_camera)
                            }
                            val battery = catalog.battery_type ?: catalog.battery_spec ?: ""
                            val charging = catalog.battery_charging ?: catalog.charging_spec ?: ""
                            SpecRow("Battery", if (charging.isNotBlank()) "$battery ($charging)" else battery.ifBlank { "-" })
                            if (!catalog.memory_spec.isNullOrBlank()) {
                                SpecRow("RAM & Storage", catalog.memory_spec)
                            }
                            if (!catalog.os_original.isNullOrBlank()) {
                                SpecRow("Original OS", catalog.os_original)
                            }
                            val dim = catalog.dimensions ?: ""
                            val weight = catalog.weight ?: ""
                            val dimCombined = if (weight.isNotBlank()) "$dim • $weight" else dim
                            if (dimCombined.isNotBlank()) {
                                SpecRow("Dimensions & Weight", dimCombined)
                            }
                        } else {
                            Text(
                                "Spesifikasi kolektif multi-sumber belum di-enrich untuk model ini.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        val live = specs.live_hardware
                        if (live != null) {
                            SpecRow("OS Version", live.os_version ?: "-")
                            SpecRow("Android SDK", live.android_sdk?.toString() ?: "-")
                            SpecRow("RAM", "${live.ram_used_mb ?: 0} MB / ${live.ram_total_mb ?: 0} MB (${((live.ram_used_mb?.toFloat() ?: 0f) / (live.ram_total_mb?.toFloat() ?: 1f) * 100).toInt()}%)")
                            SpecRow("Storage", "${live.storage_used_mb ?: 0} MB / ${live.storage_total_mb ?: 0} MB")
                            SpecRow("Battery Level", "${live.battery_percent?.toInt() ?: 0}% (${if (live.battery_charging == true) "Charging" else "Discharging"})")
                            SpecRow("Battery Temp", "${live.battery_temp_c ?: 0f} °C")
                            if (!live.hostname.isNullOrBlank()) {
                                SpecRow("Hostname", live.hostname)
                            }
                            if (!live.architecture.isNullOrBlank()) {
                                SpecRow("Architecture", live.architecture)
                            }
                            if (!live.cpu_model.isNullOrBlank()) {
                                val coresStr = if (live.cpu_cores != null) "${live.cpu_cores} Cores / ${live.cpu_logical_cores ?: live.cpu_cores} Threads" else ""
                                val freqStr = if (live.cpu_frequency_mhz != null && live.cpu_frequency_mhz > 0) "${live.cpu_frequency_mhz} MHz" else ""
                                val detail = listOf(coresStr, freqStr).filter { it.isNotBlank() }.joinToString(" • ")
                                SpecRow("CPU Model", "${live.cpu_model}${if (detail.isNotBlank()) " ($detail)" else ""}")
                            }
                            if (!live.gpu_model.isNullOrBlank()) {
                                SpecRow("GPU Adapter", live.gpu_model)
                            }
                            if (!live.uptime_formatted.isNullOrBlank()) {
                                SpecRow("System Uptime", live.uptime_formatted)
                            }
                            if (!live.kernel_version.isNullOrBlank()) {
                                SpecRow("Kernel Version", live.kernel_version)
                            }
                            if (!live.network_type.isNullOrBlank()) {
                                SpecRow("Network", live.network_type)
                            }
                            if (!live.ip_address.isNullOrBlank()) {
                                SpecRow("IP Address", live.ip_address)
                            }
                            if (live.screen_refresh_rate != null && live.screen_refresh_rate > 0f) {
                                SpecRow("Screen Refresh Rate", "${live.screen_refresh_rate} Hz")
                            }
                            if (!live.disks_info.isNullOrBlank()) {
                                SpecRow("Disks / Partitions", live.disks_info)
                            }
                            SpecRow("Last Sync", live.updated_at ?: "-")
                        } else {
                            Text(
                                "Belum ada laporan live hardware telemetry dari perangkat ini.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("Tutup")
            }
        }
    )
}

@Composable
private fun SpecRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(0.4f)
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(0.6f)
        )
    }
}
