package com.anandacahya.kusuri

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import com.anandacahya.kusuri.data.repository.KusuriRepository
import com.anandacahya.kusuri.service.KusuriTelemetryService
import com.anandacahya.kusuri.ui.auth.AuthScreen
import com.anandacahya.kusuri.ui.dashboard.MainDashboardScreen
import com.anandacahya.kusuri.ui.pairing.PairingScreen
import com.anandacahya.kusuri.ui.server.ServerManagerScreen
import com.anandacahya.kusuri.ui.server.ServerManagerViewModel
import com.anandacahya.kusuri.ui.theme.KusuriTheme
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private val serverViewModel: ServerManagerViewModel by viewModels()

    private val notificationPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { _ ->
        // Permission result handled; foreground service can show notification
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        // Request notification permission on Android 13+ for 24/7 foreground telemetry
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED
            ) {
                notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
            }
        }

        val kusuriRepository = KusuriRepository.getInstance(applicationContext)
        val serverRepository = kusuriRepository.serverRepository

        // Ensure 24/7 background telemetry service is running when active server is present
        lifecycleScope.launch {
            serverRepository.activeServerFlow.collectLatest { server ->
                if (server != null && (!server.deviceToken.isNullOrBlank() || !server.authToken.isNullOrBlank())) {
                    KusuriTelemetryService.start(applicationContext)
                }
            }
        }

        // Send telemetry immediately on app launch
        triggerAppOpenTelemetry()

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

    override fun onResume() {
        super.onResume()
        // Send telemetry immediately whenever the app is brought to foreground
        triggerAppOpenTelemetry()
    }

    private fun triggerAppOpenTelemetry() {
        val kusuriRepository = KusuriRepository.getInstance(applicationContext)
        KusuriTelemetryService.triggerNow(applicationContext)
        lifecycleScope.launch {
            val activeServer = kusuriRepository.serverRepository.activeServerFlow.firstOrNull()
            if (activeServer != null && (!activeServer.deviceToken.isNullOrBlank() || !activeServer.authToken.isNullOrBlank())) {
                kusuriRepository.wsManager.sendInitialTelemetry()
                if (!activeServer.deviceId.isNullOrBlank()) {
                    kusuriRepository.syncHardwareTelemetry(activeServer)
                }
            }
        }
    }
}
