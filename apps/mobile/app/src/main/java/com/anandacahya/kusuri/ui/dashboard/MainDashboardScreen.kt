package com.anandacahya.kusuri.ui.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.repository.KusuriRepository
import com.anandacahya.kusuri.ui.clipboard.ClipboardScreen
import com.anandacahya.kusuri.ui.commands.CommandsScreen
import com.anandacahya.kusuri.ui.devices.DevicesScreen
import com.anandacahya.kusuri.ui.files.FilesScreen
import com.anandacahya.kusuri.ui.overview.OverviewScreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainDashboardScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository,
    onNavigateToServerManager: () -> Unit,
    onNavigateToPairing: () -> Unit,
    onNavigateToAuth: () -> Unit
) {
    var selectedTabIndex by remember { mutableIntStateOf(0) }
    val isWsConnected by repository.wsManager.isConnected.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(
                            modifier = Modifier
                                .size(10.dp)
                                .background(
                                    color = if (isWsConnected) Color(0xFF2E7D32) else Color(0xFFD32F2F),
                                    shape = CircleShape
                                )
                        )
                        Spacer(modifier = Modifier.width(10.dp))
                        Column {
                            Text("Kusuri Mobile", style = MaterialTheme.typography.titleMedium)
                            Text(
                                text = activeServer?.name ?: "No Server",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                },
                actions = {
                    IconButton(onClick = onNavigateToPairing) {
                        Icon(Icons.Default.QrCodeScanner, contentDescription = "Pairing")
                    }
                    IconButton(onClick = onNavigateToAuth) {
                        Icon(Icons.Default.AccountCircle, contentDescription = "Akun")
                    }
                    IconButton(onClick = onNavigateToServerManager) {
                        Icon(Icons.Default.Settings, contentDescription = "Pengaturan Server")
                    }
                }
            )
        },
        bottomBar = {
            NavigationBar {
                NavigationBarItem(
                    selected = selectedTabIndex == 0,
                    onClick = { selectedTabIndex = 0 },
                    icon = { Icon(Icons.Default.Dashboard, contentDescription = "Overview") },
                    label = { Text("Overview") }
                )
                NavigationBarItem(
                    selected = selectedTabIndex == 1,
                    onClick = { selectedTabIndex = 1 },
                    icon = { Icon(Icons.Default.Devices, contentDescription = "Perangkat") },
                    label = { Text("Perangkat") }
                )
                NavigationBarItem(
                    selected = selectedTabIndex == 2,
                    onClick = { selectedTabIndex = 2 },
                    icon = { Icon(Icons.Default.ContentPaste, contentDescription = "Clipboard") },
                    label = { Text("Clipboard") }
                )
                NavigationBarItem(
                    selected = selectedTabIndex == 3,
                    onClick = { selectedTabIndex = 3 },
                    icon = { Icon(Icons.Default.FolderShared, contentDescription = "Berkas") },
                    label = { Text("Berkas") }
                )
                NavigationBarItem(
                    selected = selectedTabIndex == 4,
                    onClick = { selectedTabIndex = 4 },
                    icon = { Icon(Icons.Default.Terminal, contentDescription = "Perintah") },
                    label = { Text("Perintah") }
                )
            }
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            when (selectedTabIndex) {
                0 -> OverviewScreen(
                    activeServer = activeServer,
                    repository = repository,
                    onNavigateToTab = { selectedTabIndex = it },
                    onNavigateToPairing = onNavigateToPairing,
                    onNavigateToServers = onNavigateToServerManager,
                    onNavigateToAuth = onNavigateToAuth
                )
                1 -> DevicesScreen(
                    activeServer = activeServer,
                    repository = repository,
                    onNavigateToPairing = onNavigateToPairing
                )
                2 -> ClipboardScreen(
                    activeServer = activeServer,
                    repository = repository
                )
                3 -> FilesScreen(
                    activeServer = activeServer,
                    repository = repository
                )
                4 -> CommandsScreen(
                    activeServer = activeServer,
                    repository = repository
                )
            }
        }
    }
}
