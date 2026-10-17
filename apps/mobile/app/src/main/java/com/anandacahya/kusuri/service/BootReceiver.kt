package com.anandacahya.kusuri.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import com.anandacahya.kusuri.data.repository.ServerRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch

class BootReceiver : BroadcastReceiver() {
    private val TAG = "BootReceiver"

    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action
        if (action == Intent.ACTION_BOOT_COMPLETED || action == Intent.ACTION_MY_PACKAGE_REPLACED) {
            Log.d(TAG, "Device rebooted or package replaced ($action). Checking active server configuration...")
            val serverRepo = ServerRepository(context.applicationContext)
            CoroutineScope(Dispatchers.IO).launch {
                val activeServer = serverRepo.activeServerFlow.firstOrNull()
                if (activeServer != null && (!activeServer.deviceToken.isNullOrBlank() || !activeServer.authToken.isNullOrBlank())) {
                    Log.d(TAG, "Active server found (${activeServer.name}). Starting KusuriTelemetryService 24/7...")
                    KusuriTelemetryService.start(context.applicationContext)
                } else {
                    Log.d(TAG, "No active server configured. Not auto-starting service.")
                }
            }
        }
    }
}
