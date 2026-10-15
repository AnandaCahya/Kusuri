package com.anandacahya.kusuri.ui.dashboard

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.telemetry.DeviceInfoState
import com.anandacahya.kusuri.data.telemetry.DeviceTelemetryCollector

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainDashboardScreen(
    activeServer: ServerConfig?,
    onNavigateToServerManager: () -> Unit,
    onNavigateToPairing: () -> Unit
) {
    val context = LocalContext.current
    val telemetryCollector = remember { DeviceTelemetryCollector(context) }
    var deviceInfo by remember { mutableStateOf<DeviceInfoState?>(null) }
    var syncStatus by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        deviceInfo = telemetryCollector.collectDeviceInfo()
    }

    val isPaired = !activeServer?.deviceToken.isNullOrBlank()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Kusuri Dashboard") },
                actions = {
                    IconButton(onClick = {
                        deviceInfo = telemetryCollector.collectDeviceInfo()
                        syncStatus = "Telemetry refreshed locally."
                    }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh Telemetry")
                    }
                    IconButton(onClick = onNavigateToServerManager) {
                        Icon(Icons.Default.Settings, contentDescription = "Manage Servers")
                    }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Connection & Pairing Status Banner
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = if (isPaired) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.errorContainer
                ),
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = if (isPaired) Icons.Default.CheckCircle else Icons.Default.Warning,
                        contentDescription = "Status",
                        tint = if (isPaired) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error
                    )
                    Spacer(modifier = Modifier.width(12.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = if (isPaired) "Device Connected & Paired" else "Device Not Paired",
                            style = MaterialTheme.typography.titleMedium
                        )
                        Text(
                            text = if (isPaired) "Token: ${activeServer?.deviceToken?.take(10)}..." else "Tap 'Pair with 6-Digit Code' below.",
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }
            }

            // Active Server Banner Card
            Card(
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("Connected Backend Server", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(text = activeServer?.name ?: "No Server Selected", style = MaterialTheme.typography.titleLarge)
                    Text(text = activeServer?.baseUrl ?: "Configure a backend in settings", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    
                    Spacer(modifier = Modifier.height(12.dp))
                    OutlinedButton(onClick = onNavigateToServerManager) {
                        Text("Switch / Add Server (IP & DNS)")
                    }
                }
            }

            // Native Device Telemetry Card
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("Live Device Telemetry & Specs", style = MaterialTheme.typography.titleMedium)
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    deviceInfo?.let { info ->
                        Text("Model: ${info.rawModel} (${info.brand})", style = MaterialTheme.typography.bodyMedium)
                        Text("OS: ${info.androidVersion}", style = MaterialTheme.typography.bodyMedium)
                        Text("RAM: ${info.ramUsedMb} MB / ${info.ramTotalMb} MB", style = MaterialTheme.typography.bodyMedium)
                        Text("Storage Used: ${info.storageUsedMb} MB / ${info.storageTotalMb} MB", style = MaterialTheme.typography.bodyMedium)
                        Text("Battery: ${info.batteryPercent}% (${if (info.batteryCharging) "Charging" else "Discharging"}, ${info.batteryTempC}°C)", style = MaterialTheme.typography.bodyMedium)
                    } ?: Text("Loading device info...", style = MaterialTheme.typography.bodyMedium)

                    if (syncStatus != null) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(syncStatus ?: "", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                    }
                }
            }

            // Quick Actions / Pairing
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("Device Pairing", style = MaterialTheme.typography.titleMedium)
                    Spacer(modifier = Modifier.height(4.dp))
                    Text("Pair this Android device with your Kusuri account using a 6-digit code.", style = MaterialTheme.typography.bodySmall)
                    Spacer(modifier = Modifier.height(12.dp))
                    Button(onClick = onNavigateToPairing) {
                        Text(if (isPaired) "Re-Pair Device" else "Pair with 6-Digit Code")
                    }
                }
            }
        }
    }
}
