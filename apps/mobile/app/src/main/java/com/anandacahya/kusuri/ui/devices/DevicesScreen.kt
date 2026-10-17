package com.anandacahya.kusuri.ui.devices

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
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.DeviceDto
import com.anandacahya.kusuri.data.model.EnrichedDeviceSpecsResponse
import com.anandacahya.kusuri.data.model.PairingCodeResponse
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.repository.KusuriRepository
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DevicesScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository,
    onNavigateToPairing: () -> Unit
) {
    val scope = rememberCoroutineScope()
    var devices by remember { mutableStateOf<List<DeviceDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var selectedSpecs by remember { mutableStateOf<EnrichedDeviceSpecsResponse?>(null) }
    var generatedPairingCode by remember { mutableStateOf<PairingCodeResponse?>(null) }
    var deviceToDelete by remember { mutableStateOf<DeviceDto?>(null) }

    fun loadDevices() {
        if (activeServer == null) return
        scope.launch {
            isLoading = true
            errorMessage = null
            repository.getDevices(activeServer).fold(
                onSuccess = {
                    devices = it
                    isLoading = false
                },
                onFailure = {
                    errorMessage = it.localizedMessage ?: "Gagal memuat perangkat"
                    isLoading = false
                }
            )
        }
    }

    LaunchedEffect(activeServer) {
        loadDevices()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Devices & Hardware") },
                actions = {
                    IconButton(onClick = {
                        scope.launch {
                            repository.syncGoogleCatalog(activeServer)
                            loadDevices()
                        }
                    }) {
                        Icon(Icons.Default.CloudSync, contentDescription = "Sync Catalog")
                    }
                    IconButton(onClick = { loadDevices() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                }
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = {
                    scope.launch {
                        isLoading = true
                        repository.createPairingCode(activeServer).fold(
                            onSuccess = {
                                generatedPairingCode = it
                                isLoading = false
                            },
                            onFailure = {
                                errorMessage = "Gagal membuat kode: ${it.localizedMessage}"
                                isLoading = false
                            }
                        )
                    }
                },
                icon = { Icon(Icons.Default.AddLink, contentDescription = null) },
                text = { Text("Generate Pair Code") }
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (isLoading && devices.isEmpty()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            } else if (devices.isEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ) {
                    Icon(
                        imageVector = Icons.Default.DevicesOther,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        "Belum ada perangkat terhubung",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        "Gunakan tombol di bawah untuk membuat kode pairing 6-digit atau sambungkan perangkat ini.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Button(onClick = onNavigateToPairing) {
                        Text("Pair Perangkat Ini")
                    }
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

                    items(devices, key = { it.id }) { device ->
                        DeviceCard(
                            device = device,
                            onViewSpecs = {
                                scope.launch {
                                    repository.getDeviceSpecs(activeServer, device.id).fold(
                                        onSuccess = { selectedSpecs = it },
                                        onFailure = {
                                            errorMessage = "Gagal memuat specs: ${it.localizedMessage}"
                                        }
                                    )
                                }
                            },
                            onDelete = { deviceToDelete = device }
                        )
                    }
                }
            }

            // Specs Dialog
            selectedSpecs?.let { specs ->
                DeviceDetailDialog(specs = specs, onDismiss = { selectedSpecs = null })
            }

            // Generated Pairing Code Modal
            generatedPairingCode?.let { pairRes ->
                AlertDialog(
                    onDismissRequest = { generatedPairingCode = null },
                    title = { Text("Kode Pairing 6-Digit") },
                    text = {
                        Column(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Text(
                                text = pairRes.code,
                                style = MaterialTheme.typography.displayMedium,
                                color = MaterialTheme.colorScheme.primary
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                "Berlaku selama ${pairRes.expires_in_seconds / 60} menit. Masukkan kode ini di perangkat client (web, mobile, desktop) untuk menghubungkan perangkat.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    },
                    confirmButton = {
                        Button(onClick = { generatedPairingCode = null }) {
                            Text("Selesai")
                        }
                    }
                )
            }

            // Confirm Delete Dialog
            deviceToDelete?.let { dev ->
                AlertDialog(
                    onDismissRequest = { deviceToDelete = null },
                    title = { Text("Hapus Perangkat?") },
                    text = { Text("Apakah Anda yakin ingin memutuskan dan menghapus perangkat '${dev.marketing_name ?: dev.name}'?") },
                    confirmButton = {
                        TextButton(
                            onClick = {
                                val target = deviceToDelete
                                deviceToDelete = null
                                if (target != null) {
                                    scope.launch {
                                        repository.deleteDevice(activeServer, target.id)
                                        loadDevices()
                                    }
                                }
                            }
                        ) {
                            Text("Hapus", color = MaterialTheme.colorScheme.error)
                        }
                    },
                    dismissButton = {
                        TextButton(onClick = { deviceToDelete = null }) {
                            Text("Batal")
                        }
                    }
                )
            }
        }
    }
}

@Composable
fun DeviceCard(
    device: DeviceDto,
    onViewSpecs: () -> Unit,
    onDelete: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onViewSpecs() },
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val deviceIcon = when (device.device_type.lowercase()) {
                "android" -> Icons.Default.Smartphone
                "windows" -> Icons.Default.Laptop
                "linux" -> Icons.Default.Computer
                "web" -> Icons.Default.Language
                else -> Icons.Default.DevicesOther
            }

            Icon(
                imageVector = deviceIcon,
                contentDescription = device.device_type,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(36.dp)
            )

            Spacer(modifier = Modifier.width(16.dp))

            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = device.marketing_name ?: device.name,
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Surface(
                        color = if (device.is_online) Color(0xFF2E7D32) else Color(0xFF757575),
                        shape = MaterialTheme.shapes.extraSmall
                    ) {
                        Text(
                            text = if (device.is_online) "Online" else "Offline",
                            color = Color.White,
                            style = MaterialTheme.typography.labelSmall,
                            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "${device.device_type.uppercase()} • ${device.chipset ?: device.raw_model ?: "Standard Device"}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            IconButton(onClick = onViewSpecs) {
                Icon(Icons.Default.Info, contentDescription = "Specs")
            }

            IconButton(onClick = onDelete) {
                Icon(Icons.Default.DeleteOutline, contentDescription = "Delete", tint = MaterialTheme.colorScheme.error)
            }
        }
    }
}
