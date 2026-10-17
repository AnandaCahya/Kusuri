package com.anandacahya.kusuri.ui.commands

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.DeviceDto
import com.anandacahya.kusuri.data.model.RemoteCommandDto
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.remote.KusuriWsEvent
import com.anandacahya.kusuri.data.repository.KusuriRepository
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CommandsScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var commands by remember { mutableStateOf<List<RemoteCommandDto>>(emptyList()) }
    var devices by remember { mutableStateOf<List<DeviceDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var showSendDialog by remember { mutableStateOf(false) }

    // Dialog state
    var selectedTargetDeviceId by remember { mutableStateOf("") }
    var selectedCommandType by remember { mutableStateOf("ping") }
    var notifTitle by remember { mutableStateOf("Halo dari Kusuri Mobile") }
    var notifMessage by remember { mutableStateOf("Pesan remote berhasil terkirim!") }
    var customCommandText by remember { mutableStateOf("") }

    fun loadCommands() {
        if (activeServer == null) return
        scope.launch {
            isLoading = true
            errorMessage = null
            repository.getCommands(activeServer).fold(
                onSuccess = {
                    commands = it
                    isLoading = false
                },
                onFailure = {
                    errorMessage = it.localizedMessage ?: "Gagal memuat perintah"
                    isLoading = false
                }
            )
        }
    }

    fun loadDevices() {
        if (activeServer == null) return
        scope.launch {
            repository.getDevices(activeServer).fold(
                onSuccess = {
                    devices = it
                    if (selectedTargetDeviceId.isEmpty() && it.isNotEmpty()) {
                        selectedTargetDeviceId = it.first().id
                    }
                },
                onFailure = {}
            )
        }
    }

    LaunchedEffect(activeServer) {
        loadCommands()
        loadDevices()
    }

    // Listen for real-time command updates
    LaunchedEffect(repository.wsManager) {
        repository.wsManager.wsEvents.collectLatest { event ->
            if (event is KusuriWsEvent.CommandResultReceived) {
                Toast.makeText(context, "Hasil perintah remote diterima (${event.status})", Toast.LENGTH_SHORT).show()
                loadCommands()
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Remote Commands") },
                actions = {
                    IconButton(onClick = {
                        loadCommands()
                        loadDevices()
                    }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                }
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = {
                    loadDevices()
                    showSendDialog = true
                },
                icon = { Icon(Icons.Default.PlayArrow, contentDescription = null) },
                text = { Text("Kirim Perintah") }
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (isLoading && commands.isEmpty()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            } else if (commands.isEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ) {
                    Icon(
                        imageVector = Icons.Default.Terminal,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        "Belum Ada Perintah Remote",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        "Kirim perintah seperti Ping, Cek Baterai, Notifikasi, atau Kunci Layar ke perangkat terhubung Anda secara instan.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    if (errorMessage != null) {
                        item {
                            Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer)) {
                                Text(
                                    text = errorMessage ?: "",
                                    modifier = Modifier.padding(12.dp),
                                    color = MaterialTheme.colorScheme.onErrorContainer
                                )
                            }
                        }
                    }

                    items(commands, key = { it.id }) { cmd ->
                        CommandItemCard(command = cmd)
                    }
                }
            }

            // Dialog Send Command
            if (showSendDialog) {
                AlertDialog(
                    onDismissRequest = { showSendDialog = false },
                    title = { Text("Kirim Perintah Remote") },
                    text = {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            Text("Pilih Perangkat Target:", style = MaterialTheme.typography.labelMedium)
                            
                            if (devices.isEmpty()) {
                                Text(
                                    "Tidak ada perangkat terdaftar.",
                                    color = MaterialTheme.colorScheme.error,
                                    style = MaterialTheme.typography.bodySmall
                                )
                            } else {
                                var expanded by remember { mutableStateOf(false) }
                                val selectedDevice = devices.find { it.id == selectedTargetDeviceId } ?: devices.first()
                                
                                Box {
                                    OutlinedButton(
                                        onClick = { expanded = true },
                                        modifier = Modifier.fillMaxWidth()
                                    ) {
                                        Text("${selectedDevice.marketing_name ?: selectedDevice.name} (${if (selectedDevice.is_online) "Online" else "Offline"})")
                                    }
                                    DropdownMenu(
                                        expanded = expanded,
                                        onDismissRequest = { expanded = false }
                                    ) {
                                        devices.forEach { dev ->
                                            DropdownMenuItem(
                                                text = { Text("${dev.marketing_name ?: dev.name} (${if (dev.is_online) "Online" else "Offline"})") },
                                                onClick = {
                                                    selectedTargetDeviceId = dev.id
                                                    expanded = false
                                                }
                                            )
                                        }
                                    }
                                }
                            }

                            Text("Jenis Perintah:", style = MaterialTheme.typography.labelMedium)
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(6.dp)
                            ) {
                                FilterChip(
                                    selected = selectedCommandType == "ping",
                                    onClick = { selectedCommandType = "ping" },
                                    label = { Text("Ping") }
                                )
                                FilterChip(
                                    selected = selectedCommandType == "battery_check",
                                    onClick = { selectedCommandType = "battery_check" },
                                    label = { Text("Baterai") }
                                )
                                FilterChip(
                                    selected = selectedCommandType == "notification",
                                    onClick = { selectedCommandType = "notification" },
                                    label = { Text("Notifikasi") }
                                )
                            }

                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(6.dp)
                            ) {
                                FilterChip(
                                    selected = selectedCommandType == "lock_screen",
                                    onClick = { selectedCommandType = "lock_screen" },
                                    label = { Text("Kunci Layar") }
                                )
                                FilterChip(
                                    selected = selectedCommandType == "custom",
                                    onClick = { selectedCommandType = "custom" },
                                    label = { Text("Custom") }
                                )
                            }

                            if (selectedCommandType == "notification") {
                                OutlinedTextField(
                                    value = notifTitle,
                                    onValueChange = { notifTitle = it },
                                    label = { Text("Judul Notifikasi") },
                                    modifier = Modifier.fillMaxWidth()
                                )
                                OutlinedTextField(
                                    value = notifMessage,
                                    onValueChange = { notifMessage = it },
                                    label = { Text("Isi Pesan") },
                                    modifier = Modifier.fillMaxWidth()
                                )
                            } else if (selectedCommandType == "custom") {
                                OutlinedTextField(
                                    value = customCommandText,
                                    onValueChange = { customCommandText = it },
                                    label = { Text("Nama Perintah") },
                                    modifier = Modifier.fillMaxWidth()
                                )
                            }
                        }
                    },
                    confirmButton = {
                        Button(
                            onClick = {
                                val targetId = selectedTargetDeviceId.ifBlank { devices.firstOrNull()?.id }
                                if (targetId.isNullOrBlank()) {
                                    Toast.makeText(context, "Pilih perangkat target terlebih dahulu.", Toast.LENGTH_SHORT).show()
                                    return@Button
                                }

                                val finalCommand = if (selectedCommandType == "custom") customCommandText else selectedCommandType
                                val payload = when (selectedCommandType) {
                                    "notification" -> buildJsonObject {
                                        put("title", notifTitle)
                                        put("message", notifMessage)
                                    }
                                    else -> null
                                }

                                scope.launch {
                                    repository.sendCommand(activeServer, targetId, finalCommand, payload).fold(
                                        onSuccess = {
                                            Toast.makeText(context, "Perintah '$finalCommand' terkirim!", Toast.LENGTH_SHORT).show()
                                            showSendDialog = false
                                            loadCommands()
                                        },
                                        onFailure = {
                                            Toast.makeText(context, "Gagal mengirim perintah: ${it.localizedMessage}", Toast.LENGTH_LONG).show()
                                        }
                                    )
                                }
                            },
                            enabled = devices.isNotEmpty()
                        ) {
                            Text("Kirim Sekarang")
                        }
                    },
                    dismissButton = {
                        TextButton(onClick = { showSendDialog = false }) {
                            Text("Batal")
                        }
                    }
                )
            }
        }
    }
}

@Composable
fun CommandItemCard(command: RemoteCommandDto) {
    var expanded by remember { mutableStateOf(false) }

    val statusColor = when (command.status.lowercase()) {
        "completed" -> Color(0xFF2E7D32)
        "sent" -> Color(0xFF1976D2)
        "pending" -> Color(0xFFFFA000)
        "failed" -> Color(0xFFD32F2F)
        else -> Color.Gray
    }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { expanded = !expanded },
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.Default.Terminal,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = command.command.uppercase(),
                        style = MaterialTheme.typography.titleMedium
                    )
                }

                Surface(
                    color = statusColor,
                    shape = MaterialTheme.shapes.extraSmall
                ) {
                    Text(
                        text = command.status.uppercase(),
                        color = Color.White,
                        style = MaterialTheme.typography.labelSmall,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(6.dp))

            Text(
                text = "Target: ${command.target_device_name ?: command.target_device_id.take(8)} • Dari: ${command.source_device_name ?: "Mobile"}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Text(
                text = command.created_at.take(19).replace("T", " "),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            if (command.response != null) {
                Spacer(modifier = Modifier.height(8.dp))
                Surface(
                    color = MaterialTheme.colorScheme.surface,
                    shape = MaterialTheme.shapes.small,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = "Response: ${command.response}",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(8.dp),
                        maxLines = if (expanded) Int.MAX_VALUE else 2
                    )
                }
            }
        }
    }
}
