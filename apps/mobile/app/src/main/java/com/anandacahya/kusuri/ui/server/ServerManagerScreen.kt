package com.anandacahya.kusuri.ui.server

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.ServerConfig
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerManagerScreen(
    viewModel: ServerManagerViewModel,
    onNavigateBack: () -> Unit
) {
    val servers by viewModel.servers.collectAsState()
    val activeServer by viewModel.activeServer.collectAsState()
    val scope = rememberCoroutineScope()

    var showAddDialog by remember { mutableStateOf(false) }
    var testResultMsg by remember { mutableStateOf<String?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Kusuri Backend Servers") },
                navigationIcon = {
                    TextButton(onClick = onNavigateBack) { Text("Back") }
                },
                actions = {
                    IconButton(onClick = { showAddDialog = true }) {
                        Icon(Icons.Default.Add, contentDescription = "Add Server")
                    }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp)
        ) {
            Text(
                "Manage and switch between your self-hosted or cloud Kusuri backend servers (IP / DNS).",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(16.dp))

            if (testResultMsg != null) {
                Card(
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
                    modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp)
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(testResultMsg ?: "", modifier = Modifier.weight(1f))
                        TextButton(onClick = { testResultMsg = null }) { Text("Dismiss") }
                    }
                }
            }

            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(12.dp),
                modifier = Modifier.fillMaxSize()
            ) {
                items(servers) { server ->
                    ServerCard(
                        server = server,
                        isActive = server.id == activeServer?.id,
                        onSelect = { viewModel.switchActiveServer(server.id) },
                        onDelete = { viewModel.deleteServer(server.id) },
                        onTest = {
                            scope.launch {
                                testResultMsg = "Testing connection to ${server.name}..."
                                val success = viewModel.testConnection(server)
                                testResultMsg = if (success) "Successfully connected to ${server.name}!" else "Failed to connect to ${server.name}."
                            }
                        }
                    )
                }
            }
        }

        if (showAddDialog) {
            AddServerDialog(
                onDismiss = { showAddDialog = false },
                onAdd = { name, host, port, isHttps ->
                    viewModel.addServer(name, host, port, isHttps)
                    showAddDialog = false
                }
            )
        }
    }
}

@Composable
fun ServerCard(
    server: ServerConfig,
    isActive: Boolean,
    onSelect: () -> Unit,
    onDelete: () -> Unit,
    onTest: () -> Unit
) {
    val borderColor = if (isActive) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .border(width = if (isActive) 2.dp else 1.dp, color = borderColor, shape = MaterialTheme.shapes.medium)
            .clickable { onSelect() },
        elevation = CardDefaults.cardElevation(defaultElevation = if (isActive) 4.dp else 1.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = server.name,
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.weight(1f)
                )
                if (isActive) {
                    Icon(
                        imageVector = Icons.Default.CheckCircle,
                        contentDescription = "Active",
                        tint = MaterialTheme.colorScheme.primary
                    )
                }
            }

            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "URL: ${server.baseUrl}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(
                text = "WS: ${server.wsUrl}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(12.dp))

            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                OutlinedButton(onClick = onTest, modifier = Modifier.weight(1f)) {
                    Text("Test Ping")
                }
                if (!isActive) {
                    OutlinedButton(
                        onClick = onSelect,
                        modifier = Modifier.weight(1f),
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.primary)
                    ) {
                        Text("Switch Active")
                    }
                    IconButton(onClick = onDelete) {
                        Icon(Icons.Default.Delete, contentDescription = "Delete", tint = MaterialTheme.colorScheme.error)
                    }
                }
            }
        }
    }
}

@Composable
fun AddServerDialog(
    onDismiss: () -> Unit,
    onAdd: (String, String, Int, Boolean) -> Unit
) {
    var name by remember { mutableStateOf("") }
    var host by remember { mutableStateOf("") }
    var port by remember { mutableStateOf("8080") }
    var isHttps by remember { mutableStateOf(false) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add Kusuri Backend Server") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Server Name (e.g. Home VPS)") },
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = host,
                    onValueChange = { host = it },
                    label = { Text("IP Address or DNS (e.g. 192.168.1.50)") },
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = port,
                    onValueChange = { port = it },
                    label = { Text("Port (e.g. 8080)") },
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    val p = port.toIntOrNull() ?: 8080
                    if (name.isNotBlank() && host.isNotBlank()) {
                        onAdd(name, host.trim(), p, isHttps)
                    }
                }
            ) {
                Text("Add & Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        }
    )
}
