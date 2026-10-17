package com.anandacahya.kusuri.ui.overview

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.repository.KusuriRepository
import com.anandacahya.kusuri.data.telemetry.DeviceInfoState
import kotlinx.coroutines.launch

@Composable
fun OverviewScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository,
    onNavigateToTab: (Int) -> Unit,
    onNavigateToPairing: () -> Unit,
    onNavigateToServers: () -> Unit,
    onNavigateToAuth: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var deviceInfo by remember { mutableStateOf<DeviceInfoState?>(null) }
    var syncStatusText by remember { mutableStateOf<String?>(null) }
    var isSyncingTelemetry by remember { mutableStateOf(false) }

    val isWsConnected by repository.wsManager.isConnected.collectAsState()
    val isPaired = !activeServer?.deviceToken.isNullOrBlank()
    val isUserLoggedIn = !activeServer?.authToken.isNullOrBlank()

    // Counts
    var deviceCount by remember { mutableIntStateOf(0) }
    var clipboardCount by remember { mutableIntStateOf(0) }
    var fileCount by remember { mutableIntStateOf(0) }

    fun refreshTelemetry() {
        deviceInfo = repository.telemetryCollector.collectDeviceInfo()
    }

    fun loadCounts() {
        if (activeServer == null) return
        scope.launch {
            repository.getDevices(activeServer).onSuccess { deviceCount = it.size }
            repository.getClipboard(activeServer).onSuccess { clipboardCount = it.size }
            repository.getFiles(activeServer).onSuccess { fileCount = it.size }
        }
    }

    LaunchedEffect(Unit) {
        refreshTelemetry()
    }

    LaunchedEffect(activeServer) {
        loadCounts()
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Status & Connection Card
        Card(
            colors = CardDefaults.cardColors(
                containerColor = if (isPaired || isUserLoggedIn) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant
            ),
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Surface(
                            color = if (isWsConnected) Color(0xFF2E7D32) else Color(0xFFD32F2F),
                            shape = MaterialTheme.shapes.extraSmall
                        ) {
                            Text(
                                text = if (isWsConnected) "WS CONNECTED" else "WS DISCONNECTED",
                                color = Color.White,
                                style = MaterialTheme.typography.labelSmall,
                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        }
                    }

                    TextButton(onClick = onNavigateToServers) {
                        Icon(Icons.Default.Dns, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Ganti Server")
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = activeServer?.name ?: "Belum Memilih Server",
                    style = MaterialTheme.typography.titleLarge
                )
                Text(
                    text = activeServer?.baseUrl ?: "Buka Pengaturan Server untuk konfigurasi.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                Spacer(modifier = Modifier.height(12.dp))
                HorizontalDivider()
                Spacer(modifier = Modifier.height(12.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text(
                            text = if (isPaired) "Perangkat Terhubung" else "Belum Di-pair",
                            style = MaterialTheme.typography.labelLarge
                        )
                        Text(
                            text = if (isPaired)
                                activeServer?.marketingName ?: "ID: ${activeServer?.deviceId?.take(8)}..."
                            else
                                "Tap untuk pairing dengan kode 6-digit",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    if (!isPaired) {
                        Button(onClick = onNavigateToPairing) {
                            Text("Pair Sekarang")
                        }
                    }
                }
            }
        }

        // Quick Metrics Row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            MetricCard(
                title = "Perangkat",
                count = deviceCount,
                icon = Icons.Default.Devices,
                modifier = Modifier.weight(1f),
                onClick = { onNavigateToTab(1) }
            )
            MetricCard(
                title = "Clipboard",
                count = clipboardCount,
                icon = Icons.Default.ContentPaste,
                modifier = Modifier.weight(1f),
                onClick = { onNavigateToTab(2) }
            )
            MetricCard(
                title = "Berkas",
                count = fileCount,
                icon = Icons.Default.FolderShared,
                modifier = Modifier.weight(1f),
                onClick = { onNavigateToTab(3) }
            )
        }

        // Live Telemetry Card
        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Live Telemetry Perangkat HP Ini", style = MaterialTheme.typography.titleMedium)
                    IconButton(onClick = { refreshTelemetry() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh Telemetry")
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                deviceInfo?.let { info ->
                    Text("Model: ${info.rawModel} (${info.brand})", style = MaterialTheme.typography.bodyMedium)
                    Text("Sistem: ${info.androidVersion}", style = MaterialTheme.typography.bodyMedium)
                    Text("RAM: ${info.ramUsedMb} MB / ${info.ramTotalMb} MB (${((info.ramUsedMb.toFloat() / info.ramTotalMb) * 100).toInt()}%)", style = MaterialTheme.typography.bodyMedium)
                    Text("Penyimpanan: ${info.storageUsedMb} MB / ${info.storageTotalMb} MB", style = MaterialTheme.typography.bodyMedium)
                    Text("Baterai: ${info.batteryPercent.toInt()}% (${if (info.batteryCharging) "Mengisi Daya" else "Baterai"}, ${info.batteryTempC} °C)", style = MaterialTheme.typography.bodyMedium)
                } ?: CircularProgressIndicator()

                if (syncStatusText != null) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = syncStatusText ?: "",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                }

                Spacer(modifier = Modifier.height(12.dp))

                Button(
                    onClick = {
                        refreshTelemetry()
                        scope.launch {
                            isSyncingTelemetry = true
                            syncStatusText = "Mengirim status hardware ke server..."
                            // Send over WS
                            repository.wsManager.sendInitialTelemetry()
                            // Send over HTTP if paired
                            if (isPaired) {
                                repository.syncHardwareTelemetry(activeServer).fold(
                                    onSuccess = {
                                        isSyncingTelemetry = false
                                        syncStatusText = "Status hardware berhasil diperbarui di server!"
                                    },
                                    onFailure = {
                                        isSyncingTelemetry = false
                                        syncStatusText = "Terkirim via WebSocket (HTTP: ${it.localizedMessage})"
                                    }
                                )
                            } else {
                                isSyncingTelemetry = false
                                syncStatusText = "Terkirim via WebSocket."
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = !isSyncingTelemetry
                ) {
                    if (isSyncingTelemetry) {
                        CircularProgressIndicator(modifier = Modifier.size(20.dp), color = MaterialTheme.colorScheme.onPrimary)
                    } else {
                        Icon(Icons.Default.Sync, contentDescription = null)
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Sinkronkan Telemetry ke Server")
                    }
                }
            }
        }

        // Quick Actions Grid
        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text("Aksi Cepat", style = MaterialTheme.typography.titleMedium)
                Spacer(modifier = Modifier.height(12.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedButton(
                        onClick = {
                            val clipboardManager = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                            val clip = clipboardManager.primaryClip
                            if (clip != null && clip.itemCount > 0) {
                                val text = clip.getItemAt(0).text?.toString() ?: ""
                                if (text.isNotBlank()) {
                                    scope.launch {
                                        repository.createClipboard(activeServer, text)
                                        Toast.makeText(context, "Papan klip dibagikan ke server!", Toast.LENGTH_SHORT).show()
                                        loadCounts()
                                    }
                                } else {
                                    Toast.makeText(context, "Papan klip HP kosong", Toast.LENGTH_SHORT).show()
                                }
                            }
                        },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.ContentPasteGo, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Kirim Klip")
                    }

                    OutlinedButton(
                        onClick = { onNavigateToTab(3) },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.UploadFile, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Kirim File")
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedButton(
                        onClick = { onNavigateToTab(4) },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.Terminal, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Perintah Remote")
                    }

                    OutlinedButton(
                        onClick = onNavigateToAuth,
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.AccountCircle, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(if (isUserLoggedIn) "Akun Saya" else "Login Akun")
                    }
                }
            }
        }
    }
}

@Composable
fun MetricCard(
    title: String,
    count: Int,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Card(
        modifier = modifier,
        onClick = onClick,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Column(
            modifier = Modifier.padding(12.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(imageVector = icon, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
            Spacer(modifier = Modifier.height(4.dp))
            Text(text = count.toString(), style = MaterialTheme.typography.titleLarge)
            Text(text = title, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
