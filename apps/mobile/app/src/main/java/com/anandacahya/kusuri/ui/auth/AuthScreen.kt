package com.anandacahya.kusuri.ui.auth

import android.widget.Toast
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.AuthRequest
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.repository.KusuriRepository
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AuthScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository,
    onAuthSuccess: () -> Unit,
    onNavigateBack: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var selectedTab by remember { mutableStateOf(0) } // 0: Login, 1: Register
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }
    var statusMessage by remember { mutableStateOf<String?>(null) }

    val hasUserToken = !activeServer?.authToken.isNullOrBlank()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Akun Kusuri") },
                navigationIcon = {
                    TextButton(onClick = onNavigateBack) { Text("Kembali") }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(20.dp)
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = Icons.Default.AccountCircle,
                contentDescription = null,
                modifier = Modifier.size(72.dp),
                tint = MaterialTheme.colorScheme.primary
            )

            if (hasUserToken) {
                Card(
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("Status Akun Terhubung", style = MaterialTheme.typography.titleMedium)
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            "Anda saat ini login pada server: ${activeServer?.name}",
                            style = MaterialTheme.typography.bodyMedium
                        )
                        Text(
                            "Token JWT: ${activeServer?.authToken?.take(16)}...",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Button(
                            onClick = {
                                scope.launch {
                                    activeServer?.let {
                                        repository.serverRepository.updateTokens(it.id, authToken = "", deviceToken = null)
                                        Toast.makeText(context, "Logout berhasil.", Toast.LENGTH_SHORT).show()
                                    }
                                }
                            },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)
                        ) {
                            Text("Logout dari Akun")
                        }
                    }
                }
            } else {
                TabRow(selectedTabIndex = selectedTab) {
                    Tab(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        text = { Text("Masuk (Login)") }
                    )
                    Tab(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        text = { Text("Daftar Akun Baru") }
                    )
                }

                Text(
                    text = if (selectedTab == 0)
                        "Masuk dengan akun Kusuri Anda untuk mengakses file, clipboard, dan mengontrol perangkat lain."
                    else
                        "Buat akun Kusuri baru di server backend aktif (${activeServer?.name ?: "Pilih server"}).",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                OutlinedTextField(
                    value = username,
                    onValueChange = { username = it },
                    label = { Text("Username") },
                    leadingIcon = { Icon(Icons.Default.Person, contentDescription = null) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )

                OutlinedTextField(
                    value = password,
                    onValueChange = { password = it },
                    label = { Text("Password") },
                    leadingIcon = { Icon(Icons.Default.Lock, contentDescription = null) },
                    visualTransformation = PasswordVisualTransformation(),
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )

                if (statusMessage != null) {
                    Text(
                        text = statusMessage ?: "",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                }

                Button(
                    onClick = {
                        if (username.isBlank() || password.isBlank()) {
                            statusMessage = "Username dan password tidak boleh kosong."
                            return@Button
                        }
                        scope.launch {
                            isLoading = true
                            statusMessage = null
                            val req = AuthRequest(username = username.trim(), password = password)
                            val call = if (selectedTab == 0) repository.login(activeServer, req) else repository.register(activeServer, req)

                            call.fold(
                                onSuccess = { authRes ->
                                    isLoading = false
                                    Toast.makeText(context, "Berhasil masuk sebagai ${authRes.user.username}!", Toast.LENGTH_SHORT).show()
                                    // Connect WebSocket automatically with new token
                                    activeServer?.let {
                                        val updatedCfg = it.copy(authToken = authRes.token)
                                        repository.wsManager.connect(updatedCfg)
                                    }
                                    onAuthSuccess()
                                },
                                onFailure = {
                                    isLoading = false
                                    statusMessage = "Gagal: ${it.localizedMessage ?: it.message}"
                                }
                            )
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = !isLoading && username.isNotBlank() && password.isNotBlank()
                ) {
                    if (isLoading) {
                        CircularProgressIndicator(modifier = Modifier.size(24.dp), color = MaterialTheme.colorScheme.onPrimary)
                    } else {
                        Text(if (selectedTab == 0) "Masuk" else "Daftar & Masuk")
                    }
                }
            }
        }
    }
}
