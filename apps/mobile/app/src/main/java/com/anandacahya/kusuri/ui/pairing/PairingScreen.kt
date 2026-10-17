package com.anandacahya.kusuri.ui.pairing

import android.os.Build
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.*
import com.anandacahya.kusuri.data.remote.KusuriApiClientFactory
import com.anandacahya.kusuri.data.repository.ServerRepository
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PairingScreen(
    activeServer: ServerConfig?,
    serverRepository: ServerRepository,
    onPairSuccess: () -> Unit,
    onNavigateBack: () -> Unit
) {
    val scope = rememberCoroutineScope()
    val apiClientFactory = remember { KusuriApiClientFactory() }

    var pairingCode by remember { mutableStateOf("") }
    var pairingId by remember { mutableStateOf("") }
    var deviceName by remember { mutableStateOf(Build.MODEL ?: "Infinix-X6881") }
    var rawModel by remember { mutableStateOf(Build.MODEL ?: "Infinix-X6881") }
    var statusMessage by remember { mutableStateOf<String?>(null) }
    var pairedResponse by remember { mutableStateOf<PairDeviceResponse?>(null) }
    var isLoading by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Device Pairing (6-Digit Code)") },
                navigationIcon = {
                    TextButton(onClick = onNavigateBack) { Text("Back") }
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
            Text(
                "Masukan kode pairing 6-digit dari dashboard utama untuk menghubungkan perangkat ini secara permanen.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            if (activeServer == null) {
                Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer)) {
                    Text(
                        "Belum ada server backend aktif! Silakan pilih atau tambah server terlebih dahulu.",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                }
                return@Column
            }

            OutlinedTextField(
                value = pairingCode,
                onValueChange = { if (it.length <= 6) pairingCode = it },
                label = { Text("Kode Pairing 6-Digit (Wajib)") },
                placeholder = { Text("482910") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true
            )

            OutlinedTextField(
                value = pairingId,
                onValueChange = { pairingId = it },
                label = { Text("Pairing ID / UUID Sesi (Opsional)") },
                placeholder = { Text("UUID_SESI_OPSIONAL") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true
            )

            OutlinedTextField(
                value = deviceName,
                onValueChange = { deviceName = it },
                label = { Text("Nama Perangkat (Device Name)") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true
            )

            OutlinedTextField(
                value = rawModel,
                onValueChange = { rawModel = it },
                label = { Text("Raw Model (Mantan / Native Model)") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true
            )

            if (statusMessage != null) {
                Text(
                    text = statusMessage ?: "",
                    color = if (statusMessage!!.contains("Berhasil") || statusMessage!!.contains("Success")) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error
                )
            }

            pairedResponse?.let { res ->
                Card(
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("Detail Pairing Berhasil:", style = MaterialTheme.typography.titleMedium)
                        Spacer(modifier = Modifier.height(4.dp))
                        Text("Marketing Name: ${res.marketing_name ?: res.device_name}")
                        Text("Chipset: ${res.chipset ?: "Unresolved"}")
                        Text("Device ID: ${res.device_id}")
                        Text("Device Token: ${res.device_token.take(15)}...")
                    }
                }
            }

            Button(
                onClick = {
                    if (pairingCode.length != 6) {
                        statusMessage = "Kode pairing harus 6 digit angka."
                        return@Button
                    }
                    scope.launch {
                        isLoading = true
                        statusMessage = "Menghubungkan ke server ${activeServer.baseUrl}..."
                        try {
                            val apiService = apiClientFactory.createService(activeServer)
                            val request = PairDeviceRequest(
                                pairing_code = pairingCode,
                                pairing_id = pairingId.ifBlank { null },
                                device_name = deviceName.ifBlank { rawModel },
                                device_type = "android",
                                raw_model = rawModel
                            )
                            val response = apiService.pairDevice(request)
                            pairedResponse = response
                            
                            serverRepository.updatePairingInfo(
                                serverId = activeServer.id,
                                deviceToken = response.device_token,
                                deviceId = response.device_id,
                                marketingName = response.marketing_name ?: response.device_name,
                                chipset = response.chipset
                            )

                            if (!response.auth_token.isNullOrBlank()) {
                                serverRepository.updateTokens(
                                    serverId = activeServer.id,
                                    authToken = response.auth_token,
                                    deviceToken = response.device_token
                                )
                            }

                            statusMessage = "Berhasil Paired sebagai '${response.marketing_name ?: response.device_name}'!"
                            isLoading = false
                            kotlinx.coroutines.delay(1500)
                            onPairSuccess()
                        } catch (e: Exception) {
                            isLoading = false
                            statusMessage = "Pairing Gagal: ${e.localizedMessage ?: e.message}"
                        }
                    }
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = !isLoading && pairingCode.length == 6
            ) {
                if (isLoading) {
                    CircularProgressIndicator(modifier = Modifier.size(24.dp), color = MaterialTheme.colorScheme.onPrimary)
                } else {
                    Text("Pair Device Now")
                }
            }
        }
    }
}
