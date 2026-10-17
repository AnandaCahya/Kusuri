package com.anandacahya.kusuri

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.lifecycle.lifecycleScope
import com.anandacahya.kusuri.data.repository.KusuriRepository
import com.anandacahya.kusuri.data.repository.ServerRepository
import com.anandacahya.kusuri.ui.auth.AuthScreen
import com.anandacahya.kusuri.ui.dashboard.MainDashboardScreen
import com.anandacahya.kusuri.ui.pairing.PairingScreen
import com.anandacahya.kusuri.ui.server.ServerManagerScreen
import com.anandacahya.kusuri.ui.server.ServerManagerViewModel
import com.anandacahya.kusuri.ui.theme.KusuriTheme
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private val serverViewModel: ServerManagerViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val serverRepository = ServerRepository(applicationContext)
        val kusuriRepository = KusuriRepository(applicationContext, serverRepository, lifecycleScope)

        // Automatically connect or reconnect WebSocket whenever active server changes
        lifecycleScope.launch {
            serverRepository.activeServerFlow.collectLatest { server ->
                if (server != null && (!server.deviceToken.isNullOrBlank() || !server.authToken.isNullOrBlank())) {
                    kusuriRepository.wsManager.connect(server)
                } else {
                    kusuriRepository.wsManager.disconnect()
                }
            }
        }

        setContent {
            KusuriTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    var currentScreen by remember { mutableStateOf("dashboard") }
                    val activeServer by serverViewModel.activeServer.collectAsState()

                    when (currentScreen) {
                        "dashboard" -> MainDashboardScreen(
                            activeServer = activeServer,
                            repository = kusuriRepository,
                            onNavigateToServerManager = { currentScreen = "servers" },
                            onNavigateToPairing = { currentScreen = "pairing" },
                            onNavigateToAuth = { currentScreen = "auth" }
                        )
                        "servers" -> ServerManagerScreen(
                            viewModel = serverViewModel,
                            onNavigateBack = { currentScreen = "dashboard" }
                        )
                        "pairing" -> PairingScreen(
                            activeServer = activeServer,
                            serverRepository = serverRepository,
                            onPairSuccess = { currentScreen = "dashboard" },
                            onNavigateBack = { currentScreen = "dashboard" }
                        )
                        "auth" -> AuthScreen(
                            activeServer = activeServer,
                            repository = kusuriRepository,
                            onAuthSuccess = { currentScreen = "dashboard" },
                            onNavigateBack = { currentScreen = "dashboard" }
                        )
                    }
                }
            }
        }
    }
}
