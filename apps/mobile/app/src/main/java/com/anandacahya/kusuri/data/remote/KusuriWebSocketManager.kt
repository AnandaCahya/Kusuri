package com.anandacahya.kusuri.data.remote

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.os.Build
import android.util.Log
import android.widget.Toast
import com.anandacahya.kusuri.data.model.ClipboardItemDto
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.telemetry.BrandDeviceResolver
import com.anandacahya.kusuri.data.telemetry.DeviceInfoState
import com.anandacahya.kusuri.data.telemetry.DeviceTelemetryCollector
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import okhttp3.*
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlin.math.min
import kotlin.math.pow
import kotlin.random.Random

sealed class KusuriWsEvent {
    data class ClipboardUpdated(val item: ClipboardItemDto) : KusuriWsEvent()
    data class FileAvailable(
        val fileId: String,
        val originalFilename: String,
        val fileSize: Long,
        val mimeType: String,
        val downloadUrl: String,
        val sourceDeviceId: String?
    ) : KusuriWsEvent()
    data class DeviceConnected(val deviceId: String, val name: String, val deviceType: String) : KusuriWsEvent()
    data class DeviceDisconnected(val deviceId: String) : KusuriWsEvent()
    data class CommandReceived(val commandId: String, val command: String, val payload: JSONObject?) : KusuriWsEvent()
    data class CommandResultReceived(val commandId: String, val status: String, val response: JSONObject?) : KusuriWsEvent()
}

class KusuriWebSocketManager(
    private val context: Context,
    externalScope: CoroutineScope? = null
) {
    private val TAG = "KusuriWS"
    // Use an application-lifecycled supervisor scope so WebSocket persists regardless of UI lifecycle
    private val scope = externalScope ?: CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var webSocket: WebSocket? = null
    private val client = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .pingInterval(25, TimeUnit.SECONDS)
        .retryOnConnectionFailure(true)
        .build()

    private val telemetryCollector = DeviceTelemetryCollector(context)
    private val brandInfo = BrandDeviceResolver.resolve(context)

    private val _isConnected = MutableStateFlow(false)
    val isConnected: StateFlow<Boolean> = _isConnected

    private val _wsEvents = MutableSharedFlow<KusuriWsEvent>(extraBufferCapacity = 64)
    val wsEvents: SharedFlow<KusuriWsEvent> = _wsEvents

    private var pingJob: Job? = null
    private var reconnectJob: Job? = null
    private var activeConfig: ServerConfig? = null
    private var shouldAutoReconnect = false
    private var reconnectAttempt = 0

    private val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
    private val networkCallback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) {
            Log.d(TAG, "Network connection restored. Checking WebSocket status...")
            if (shouldAutoReconnect && !_isConnected.value) {
                scope.launch {
                    delay(1000) // Brief grace period for network route stabilisation
                    Log.d(TAG, "Triggering immediate reconnect on network available...")
                    reconnectJob?.cancel()
                    connectInternal()
                }
            }
        }
    }

    init {
        try {
            connectivityManager?.registerDefaultNetworkCallback(networkCallback)
        } catch (e: Exception) {
            Log.w(TAG, "Could not register default network callback: ${e.message}")
        }
    }

    fun connect(serverConfig: ServerConfig) {
        shouldAutoReconnect = true
        reconnectAttempt = 0
        activeConfig = serverConfig
        connectInternal()
    }

    @Synchronized
    private fun connectInternal() {
        val serverConfig = activeConfig ?: return
        val token = serverConfig.deviceToken ?: serverConfig.authToken
        if (token.isNullOrBlank()) {
            Log.w(TAG, "Cannot connect: Token is missing for server ${serverConfig.name}")
            return
        }

        // Clean up previous connection if any
        stopHeartbeat()
        try {
            webSocket?.close(1000, "Reconnecting")
        } catch (_: Exception) {}
        webSocket = null

        val wsUrl = "${serverConfig.wsUrl}?token=$token"
        Log.d(TAG, "Connecting to $wsUrl (Attempt $reconnectAttempt)...")

        val request = Request.Builder().url(wsUrl).build()
        webSocket = client.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                Log.d(TAG, "WebSocket Connected successfully to ${serverConfig.name}")
                _isConnected.value = true
                reconnectAttempt = 0
                reconnectJob?.cancel()
                startHeartbeat()
                // Automatically send native hardware telemetry upon connection
                sendInitialTelemetry()
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                Log.d(TAG, "WS Message: $text")
                handleIncomingMessage(text)
            }

            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                Log.d(TAG, "WS Closing: $code / $reason")
                _isConnected.value = false
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                Log.d(TAG, "WS Closed: $reason")
                _isConnected.value = false
                stopHeartbeat()
                scheduleReconnect()
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                Log.e(TAG, "WS Failure: ${t.message}")
                _isConnected.value = false
                stopHeartbeat()
                scheduleReconnect()
            }
        })
    }

    private fun scheduleReconnect() {
        if (!shouldAutoReconnect) return
        reconnectJob?.cancel()
        reconnectJob = scope.launch {
            reconnectAttempt++
            // Exponential backoff: 2s, 3s, 4.5s, 6.75s, ... max 60s
            val baseDelay = (2000L * 1.5.pow((reconnectAttempt - 1).coerceAtMost(8))).toLong()
            val cappedDelay = min(60_000L, baseDelay)
            val jitter = Random.nextLong(-500, 500)
            val finalDelay = (cappedDelay + jitter).coerceAtLeast(1500L)

            Log.d(TAG, "Auto-retry scheduled in ${finalDelay}ms (Attempt #$reconnectAttempt)")
            delay(finalDelay)

            if (shouldAutoReconnect && !_isConnected.value) {
                Log.d(TAG, "Executing scheduled reconnect to ${activeConfig?.name}...")
                connectInternal()
            }
        }
    }

    private fun handleIncomingMessage(text: String) {
        try {
            val json = JSONObject(text)
            val type = json.optString("type")
            val payload = json.optJSONObject("payload")

            when (type) {
                "pong" -> {
                    // Pong received
                }
                "clipboard_updated" -> {
                    payload?.let {
                        val userId = if (it.has("user_id") && !it.isNull("user_id")) it.getString("user_id") else null
                        val srcDevId = if (it.has("source_device_id") && !it.isNull("source_device_id")) it.getString("source_device_id") else null
                        val srcDevName = if (it.has("source_device_name") && !it.isNull("source_device_name")) it.getString("source_device_name") else null

                        val item = ClipboardItemDto(
                            id = it.optString("id"),
                            user_id = userId,
                            source_device_id = srcDevId,
                            source_device_name = srcDevName,
                            content = it.optString("content"),
                            content_type = it.optString("content_type", "text/plain"),
                            created_at = it.optString("created_at")
                        )
                        scope.launch { _wsEvents.emit(KusuriWsEvent.ClipboardUpdated(item)) }
                    }
                }
                "file_available" -> {
                    payload?.let {
                        val srcDevId = if (it.has("source_device_id") && !it.isNull("source_device_id")) it.getString("source_device_id") else null
                        val event = KusuriWsEvent.FileAvailable(
                            fileId = it.optString("file_id"),
                            originalFilename = it.optString("original_filename"),
                            fileSize = it.optLong("file_size"),
                            mimeType = it.optString("mime_type"),
                            downloadUrl = it.optString("download_url"),
                            sourceDeviceId = srcDevId
                        )
                        scope.launch { _wsEvents.emit(event) }
                    }
                }
                "device_connected" -> {
                    payload?.let {
                        val event = KusuriWsEvent.DeviceConnected(
                            deviceId = it.optString("device_id"),
                            name = it.optString("name"),
                            deviceType = it.optString("device_type")
                        )
                        scope.launch { _wsEvents.emit(event) }
                    }
                }
                "device_disconnected" -> {
                    payload?.let {
                        val event = KusuriWsEvent.DeviceDisconnected(
                            deviceId = it.optString("device_id")
                        )
                        scope.launch { _wsEvents.emit(event) }
                    }
                }
                "execute_command" -> {
                    payload?.let {
                        val commandId = it.optString("command_id")
                        val command = it.optString("command")
                        val cmdPayload = it.optJSONObject("payload")
                        scope.launch {
                            _wsEvents.emit(KusuriWsEvent.CommandReceived(commandId, command, cmdPayload))
                            // Automatically execute standard commands on Android client
                            executeRemoteCommandLocally(commandId, command, cmdPayload)
                        }
                    }
                }
                "command_result_received" -> {
                    payload?.let {
                        val event = KusuriWsEvent.CommandResultReceived(
                            commandId = it.optString("command_id"),
                            status = it.optString("status"),
                            response = it.optJSONObject("response")
                        )
                        scope.launch { _wsEvents.emit(event) }
                    }
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing incoming WS event: ${e.message}", e)
        }
    }

    private fun executeRemoteCommandLocally(commandId: String, command: String, payload: JSONObject?) {
        scope.launch(Dispatchers.Main) {
            try {
                when (command.lowercase()) {
                    "ping" -> {
                        val responseJson = JSONObject().apply {
                            put("pong", true)
                            put("device_model", brandInfo.rawModel)
                            put("brand", brandInfo.brand)
                            put("marketing_name", brandInfo.marketingName ?: "")
                            put("display_name", brandInfo.bestDisplayName)
                            put("manufacturer", brandInfo.manufacturer)
                            put("android_version", Build.VERSION.RELEASE)
                            put("kernel_version", brandInfo.kernelVersion ?: "")
                            put("timestamp", System.currentTimeMillis())
                        }
                        sendCommandResult(commandId, "completed", responseJson)
                    }
                    "battery_check" -> {
                        val info = telemetryCollector.collectDeviceInfo()
                        val responseJson = JSONObject().apply {
                            put("battery_percent", info.batteryPercent)
                            put("battery_charging", info.batteryCharging)
                            put("battery_temp_c", info.batteryTempC)
                            put("battery_health", info.batteryHealth)
                            put("timestamp", System.currentTimeMillis())
                        }
                        sendCommandResult(commandId, "completed", responseJson)
                    }
                    "notification" -> {
                        val title = payload?.optString("title", "Kusuri Remote Alert") ?: "Kusuri Remote Alert"
                        val message = payload?.optString("message", "Perintah remote diterima!") ?: "Perintah remote diterima!"
                        Toast.makeText(context, "$title: $message", Toast.LENGTH_LONG).show()

                        val responseJson = JSONObject().apply {
                            put("displayed", true)
                            put("title", title)
                            put("message", message)
                            put("timestamp", System.currentTimeMillis())
                        }
                        sendCommandResult(commandId, "completed", responseJson)
                    }
                    "lock_screen" -> {
                        Toast.makeText(context, "Perintah remote: Kunci Layar diterima.", Toast.LENGTH_SHORT).show()
                        val responseJson = JSONObject().apply {
                            put("action", "lock_screen")
                            put("acknowledged", true)
                        }
                        sendCommandResult(commandId, "completed", responseJson)
                    }
                    else -> {
                        // Generic command acknowledgement
                        Toast.makeText(context, "Remote Command: $command", Toast.LENGTH_SHORT).show()
                        val responseJson = JSONObject().apply {
                            put("status", "executed")
                            put("command", command)
                            put("received_payload", payload ?: JSONObject())
                        }
                        sendCommandResult(commandId, "completed", responseJson)
                    }
                }
            } catch (e: Exception) {
                val errorJson = JSONObject().apply {
                    put("error", e.message ?: "Execution failed")
                }
                sendCommandResult(commandId, "failed", errorJson)
            }
        }
    }

    fun sendCommandResult(commandId: String, status: String, response: JSONObject) {
        val json = JSONObject().apply {
            put("type", "command_result")
            put("payload", JSONObject().apply {
                put("command_id", commandId)
                put("status", status)
                put("response", response)
            })
        }
        sendMessage(json.toString())
    }

    fun sendClipboardSync(content: String, contentType: String = "text/plain") {
        val json = JSONObject().apply {
            put("type", "clipboard_sync")
            put("payload", JSONObject().apply {
                put("content", content)
                put("content_type", contentType)
            })
        }
        sendMessage(json.toString())
    }

    fun sendInitialTelemetry() {
        scope.launch(Dispatchers.IO) {
            try {
                val info = telemetryCollector.collectDeviceInfo()
                sendTelemetryData(info)
                Log.d(TAG, "Sent hardware telemetry over WS successfully.")
            } catch (e: Exception) {
                Log.e(TAG, "Failed sending telemetry over WS: ${e.message}")
            }
        }
    }

    fun sendTelemetryData(info: DeviceInfoState): Boolean {
        return try {
            val modelForTelemetry = if (!info.marketingName.isNullOrBlank()) {
                info.displayName
            } else {
                info.rawModel
            }

            val json = JSONObject().apply {
                put("type", "hardware_telemetry")
                put("payload", JSONObject().apply {
                    put("raw_model", modelForTelemetry)
                    put("ram_total_mb", info.ramTotalMb)
                    put("ram_used_mb", info.ramUsedMb)
                    put("ram_free_mb", info.ramFreeMb)
                    put("storage_total_mb", info.storageTotalMb)
                    put("storage_used_mb", info.storageUsedMb)
                    put("storage_free_mb", info.storageFreeMb)
                    put("battery_percent", info.batteryPercent)
                    put("battery_charging", info.batteryCharging)
                    put("battery_temp_c", info.batteryTempC)
                    put("battery_health", info.batteryHealth)
                    put("os_version", info.androidVersion)
                    put("android_sdk", info.sdkInt)
                    info.kernelVersion?.let { put("kernel_version", it) }
                    info.screenResolution.takeIf { it.isNotBlank() }?.let { put("screen_resolution", it) }
                    put("screen_refresh_rate", info.screenRefreshRate)
                    info.ipAddress?.let { put("ip_address", it) }
                    info.networkType?.let { put("network_type", it) }
                })
            }
            sendMessage(json.toString())
        } catch (e: Exception) {
            Log.e(TAG, "Error creating telemetry payload: ${e.message}")
            false
        }
    }

    fun sendRemoteCommand(targetDeviceId: String, command: String, payload: JSONObject? = null) {
        val json = JSONObject().apply {
            put("type", "send_command")
            put("payload", JSONObject().apply {
                put("target_device_id", targetDeviceId)
                put("command", command)
                put("payload", payload ?: JSONObject())
            })
        }
        sendMessage(json.toString())
    }

    fun sendMessage(jsonPayload: String): Boolean {
        val sent = webSocket?.send(jsonPayload) ?: false
        if (!sent) {
            Log.w(TAG, "Message sending failed or WS not connected.")
        }
        return sent
    }

    private fun startHeartbeat() {
        pingJob?.cancel()
        pingJob = scope.launch {
            while (isActive) {
                delay(25_000)
                val pingJson = JSONObject().put("type", "ping").toString()
                webSocket?.send(pingJson)
            }
        }
    }

    private fun stopHeartbeat() {
        pingJob?.cancel()
        pingJob = null
    }

    fun disconnect() {
        shouldAutoReconnect = false
        reconnectJob?.cancel()
        reconnectJob = null
        stopHeartbeat()
        try {
            webSocket?.close(1000, "Client disconnect")
        } catch (_: Exception) {}
        webSocket = null
        _isConnected.value = false
    }
}
