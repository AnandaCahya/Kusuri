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
import com.anandacahya.kusuri.data.repository.ServerRepository
import com.anandacahya.kusuri.ui.dashboard.MainDashboardScreen
import com.anandacahya.kusuri.ui.pairing.PairingScreen
import com.anandacahya.kusuri.ui.server.ServerManagerScreen
import com.anandacahya.kusuri.ui.server.ServerManagerViewModel
import com.anandacahya.kusuri.ui.theme.KusuriTheme

class MainActivity : ComponentActivity() {
    private val serverViewModel: ServerManagerViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val serverRepository = ServerRepository(applicationContext)

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
                            onNavigateToServerManager = { currentScreen = "servers" },
                            onNavigateToPairing = { currentScreen = "pairing" }
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
                    }
                }
            }
        }
    }
}
