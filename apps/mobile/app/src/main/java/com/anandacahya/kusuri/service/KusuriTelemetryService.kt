package com.anandacahya.kusuri.service

import android.app.*
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ServiceInfo
import android.os.BatteryManager
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import com.anandacahya.kusuri.MainActivity
import com.anandacahya.kusuri.R
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.repository.KusuriRepository
import com.anandacahya.kusuri.data.telemetry.BrandDeviceResolver
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.collectLatest

class KusuriTelemetryService : Service() {

    private val TAG = "KusuriTelemetrySvc"
    private val serviceJob = SupervisorJob()
    private val serviceScope = CoroutineScope(Dispatchers.IO + serviceJob)

    private lateinit var repository: KusuriRepository
    private var telemetryJob: Job? = null
    private var activeServer: ServerConfig? = null

    // Tracking last sent state for UI notification display
    private var lastSentBatteryPct: Float = -1f
    private var lastSentIsCharging: Boolean = false
    private var lastSentTempC: Float = -1f
    private var lastSentRamUsedMb: Long = -1L
    private var lastSentTimestamp: Long = 0L

    companion object {
        const val CHANNEL_ID = "kusuri_telemetry_channel"
        const val NOTIFICATION_ID = 1001

        const val ACTION_START = "com.anandacahya.kusuri.action.START_TELEMETRY"
        const val ACTION_STOP = "com.anandacahya.kusuri.action.STOP_TELEMETRY"
        const val ACTION_TRIGGER_NOW = "com.anandacahya.kusuri.action.TRIGGER_NOW"

        fun start(context: Context) {
            val intent = Intent(context, KusuriTelemetryService::class.java).apply {
                action = ACTION_START
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            val intent = Intent(context, KusuriTelemetryService::class.java).apply {
                action = ACTION_STOP
            }
            context.startService(intent)
        }

        fun triggerNow(context: Context) {
            val intent = Intent(context, KusuriTelemetryService::class.java).apply {
                action = ACTION_TRIGGER_NOW
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }
    }

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "Creating 24/7 KusuriTelemetryService...")
        repository = KusuriRepository.getInstance(applicationContext)
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            Log.i(TAG, "Stopping KusuriTelemetryService on user request.")
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
            return START_NOT_STICKY
        }

        // Start as foreground service immediately with low-importance ongoing notification
        val notification = buildNotification("Menginisialisasi live telemetry 24 jam...")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        observeServerAndTelemetry()

        // Handle immediate trigger request (e.g. when app is opened)
        if (intent?.action == ACTION_TRIGGER_NOW) {
            Log.d(TAG, "Immediate telemetry trigger requested.")
            serviceScope.launch {
                activeServer?.let { evaluateAndSendTelemetry(it) }
            }
        }

        // START_STICKY tells the OS to restart this service if killed by memory pressure
        return START_STICKY
    }

    private fun observeServerAndTelemetry() {
        serviceScope.launch {
            repository.activeServerFlow.collectLatest { server ->
                activeServer = server
                if (server != null && (!server.deviceToken.isNullOrBlank() || !server.authToken.isNullOrBlank())) {
                    Log.i(TAG, "Active server configured: ${server.name}. Starting telemetry loop...")
                    updateNotification("Aktif • Terhubung ke ${server.name}")
                    repository.wsManager.connect(server)
                    startTelemetryLoop()
                } else {
                    Log.w(TAG, "No active server configured. Pausing telemetry loop...")
                    updateNotification("Siaga • Menunggu konfigurasi server aktif")
                    repository.wsManager.disconnect()
                    stopTelemetryLoop()
                }
            }
        }
    }

    private fun startTelemetryLoop() {
        telemetryJob?.cancel()
        telemetryJob = serviceScope.launch {
            val powerManager = getSystemService(Context.POWER_SERVICE) as? PowerManager
            val brandInfo = BrandDeviceResolver.resolve(applicationContext)

            Log.i(TAG, "Telemetry engine initialized for: ${brandInfo.bestDisplayName}")

            while (isActive) {
                try {
                    val server = activeServer
                    if (server != null) {
                        evaluateAndSendTelemetry(server)
                    }

                    // Periodic interval configuration:
                    // - 30 seconds when screen is ON or device is plugged into charger
                    // - 60 seconds interval in the background process
                    val isInteractive = powerManager?.isInteractive ?: true
                    val isCharging = checkIsCharging()

                    val sleepDelayMs = when {
                        isCharging -> 30_000L      // 30 seconds while charging
                        isInteractive -> 30_000L   // 30 seconds while screen is on
                        else -> 60_000L            // 60 seconds periodic interval in background process
                    }

                    delay(sleepDelayMs)
                } catch (e: CancellationException) {
                    break
                } catch (e: Exception) {
                    Log.e(TAG, "Error in telemetry loop: ${e.message}")
                    delay(15_000L) // Auto-retry in 15 seconds on error
                }
            }
        }
    }

    private suspend fun evaluateAndSendTelemetry(server: ServerConfig) {
        val collector = repository.telemetryCollector
        val info = collector.collectDeviceInfo()
        val now = System.currentTimeMillis()

        Log.d(TAG, "Transmitting interval telemetry: Battery=${info.batteryPercent}%, Temp=${info.batteryTempC}C, Charging=${info.batteryCharging}, RAM=${info.ramUsedMb}MB")

        // 1. First attempt transmission via persistent WebSocket
        var sent = false
        if (repository.wsManager.isConnected.value) {
            sent = repository.wsManager.sendTelemetryData(info)
        }

        // 2. If WS is disconnected or sending failed, auto-retry via HTTP API
        if (!sent) {
            Log.d(TAG, "WebSocket offline, auto-retrying telemetry via HTTP sync...")
            val httpResult = repository.syncHardwareTelemetry(server)
            if (httpResult.isSuccess) {
                sent = true
                Log.d(TAG, "Telemetry synced successfully via HTTP fallback.")
            } else {
                Log.w(TAG, "HTTP telemetry sync notice: ${httpResult.exceptionOrNull()?.message}")
            }
        }

        if (sent) {
            lastSentBatteryPct = info.batteryPercent
            lastSentIsCharging = info.batteryCharging
            lastSentTempC = info.batteryTempC
            lastSentRamUsedMb = info.ramUsedMb
            lastSentTimestamp = now

            updateNotification("Aktif • ${info.displayName} (${info.batteryPercent.toInt()}%${if (info.batteryCharging) "⚡" else ""})")
        }
    }

    private fun checkIsCharging(): Boolean {
        val intent = registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val status = intent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        return status == BatteryManager.BATTERY_STATUS_CHARGING || status == BatteryManager.BATTERY_STATUS_FULL
    }

    private fun stopTelemetryLoop() {
        telemetryJob?.cancel()
        telemetryJob = null
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Kusuri Live Telemetry 24/7",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Layanan latar belakang Kusuri untuk live telemetry hemat daya"
                setShowBadge(false)
                enableLights(false)
                enableVibration(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(statusText: String): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java).apply {
                flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            },
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Kusuri Service Aktif")
            .setContentText(statusText)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setContentIntent(pendingIntent)
            .build()
    }

    private fun updateNotification(statusText: String) {
        try {
            val manager = getSystemService(NotificationManager::class.java)
            manager?.notify(NOTIFICATION_ID, buildNotification(statusText))
        } catch (_: Exception) {}
    }

    override fun onDestroy() {
        Log.i(TAG, "KusuriTelemetryService onDestroy triggered.")
        stopTelemetryLoop()
        serviceJob.cancel()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
