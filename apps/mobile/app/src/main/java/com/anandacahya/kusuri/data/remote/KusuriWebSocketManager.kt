package com.anandacahya.kusuri.data.remote

import android.content.Context
import android.os.Build
import android.util.Log
import android.widget.Toast
import com.anandacahya.kusuri.data.model.ClipboardItemDto
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.telemetry.DeviceTelemetryCollector
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import okhttp3.*
import org.json.JSONObject
import java.util.concurrent.TimeUnit

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
    private val scope: CoroutineScope
) {
    private val TAG = "KusuriWS"
    private var webSocket: WebSocket? = null
    private val client = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .pingInterval(25, TimeUnit.SECONDS)
        .build()

    private val telemetryCollector = DeviceTelemetryCollector(context)

    private val _isConnected = MutableStateFlow(false)
    val isConnected: StateFlow<Boolean> = _isConnected

    private val _wsEvents = MutableSharedFlow<KusuriWsEvent>(extraBufferCapacity = 64)
    val wsEvents: SharedFlow<KusuriWsEvent> = _wsEvents

    private var pingJob: Job? = null
    private var activeConfig: ServerConfig? = null

    fun connect(serverConfig: ServerConfig) {
        disconnect()
        activeConfig = serverConfig
        val token = serverConfig.deviceToken ?: serverConfig.authToken
        if (token.isNullOrBlank()) {
            Log.w(TAG, "Cannot connect: Token is missing for server ${serverConfig.name}")
            return
        }

        val wsUrl = "${serverConfig.wsUrl}?token=$token"
        Log.d(TAG, "Connecting to $wsUrl ...")

        val request = Request.Builder().url(wsUrl).build()
        webSocket = client.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                Log.d(TAG, "WebSocket Connected successfully to ${serverConfig.name}")
                _isConnected.value = true
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
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                Log.e(TAG, "WS Failure: ${t.message}")
                _isConnected.value = false
                stopHeartbeat()
            }
        })
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
                            put("device_model", Build.MODEL)
                            put("manufacturer", Build.MANUFACTURER)
                            put("android_version", Build.VERSION.RELEASE)
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
                val json = JSONObject().apply {
                    put("type", "hardware_telemetry")
                    put("payload", JSONObject().apply {
                        put("raw_model", info.rawModel)
                        put("ram_total_mb", info.ramTotalMb)
                        put("ram_used_mb", info.ramUsedMb)
                        put("storage_total_mb", info.storageTotalMb)
                        put("storage_used_mb", info.storageUsedMb)
                        put("battery_percent", info.batteryPercent)
                        put("battery_charging", info.batteryCharging)
                        put("battery_temp_c", info.batteryTempC)
                        put("os_version", info.androidVersion)
                        put("android_sdk", info.sdkInt)
                    })
                }
                sendMessage(json.toString())
                Log.d(TAG, "Sent hardware telemetry over WS successfully.")
            } catch (e: Exception) {
                Log.e(TAG, "Failed sending telemetry over WS: ${e.message}")
            }
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

    fun sendMessage(jsonPayload: String) {
        val sent = webSocket?.send(jsonPayload) ?: false
        if (!sent) {
            Log.w(TAG, "Message sending failed or WS not connected.")
        }
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
        stopHeartbeat()
        try {
            webSocket?.close(1000, "Client disconnect")
        } catch (_: Exception) {}
        webSocket = null
        _isConnected.value = false
    }
}
