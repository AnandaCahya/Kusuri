package com.anandacahya.kusuri.data.remote

import android.util.Log
import com.anandacahya.kusuri.data.model.ServerConfig
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import okhttp3.*
import org.json.JSONObject

class KusuriWebSocketManager(
    private val scope: CoroutineScope
) {
    private var webSocket: WebSocket? = null
    private val client = OkHttpClient.Builder().build()
    
    private val _incomingEvents = MutableSharedFlow<String>()
    val incomingEvents: SharedFlow<String> = _incomingEvents

    private var pingJob: Job? = null

    fun connect(serverConfig: ServerConfig) {
        disconnect()
        val token = serverConfig.deviceToken ?: serverConfig.authToken ?: ""
        val wsUrl = "${serverConfig.wsUrl}?token=$token"
        
        val request = Request.Builder().url(wsUrl).build()
        webSocket = client.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                Log.d("KusuriWS", "Connected to ${serverConfig.wsUrl}")
                startHeartbeat()
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                Log.d("KusuriWS", "Received: $text")
                scope.launch {
                    _incomingEvents.emit(text)
                }
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                Log.d("KusuriWS", "Closed: $reason")
                stopHeartbeat()
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                Log.e("KusuriWS", "Failure: ${t.message}", t)
                stopHeartbeat()
            }
        })
    }

    fun sendMessage(jsonPayload: String) {
        webSocket?.send(jsonPayload)
    }

    fun sendClipboardSync(content: String) {
        val json = JSONObject().apply {
            put("type", "clipboard_sync")
            put("payload", JSONObject().apply {
                put("content", content)
                put("content_type", "text/plain")
            })
        }
        sendMessage(json.toString())
    }

    fun sendHardwareTelemetry(rawModel: String, batteryPercent: Float, isCharging: Boolean, ramUsed: Long, ramTotal: Long) {
        val json = JSONObject().apply {
            put("type", "hardware_telemetry")
            put("payload", JSONObject().apply {
                put("raw_model", rawModel)
                put("battery_percent", batteryPercent)
                put("battery_charging", isCharging)
                put("ram_used_mb", ramUsed)
                put("ram_total_mb", ramTotal)
                put("storage_total_mb", 262144)
                put("storage_used_mb", 95200)
                put("battery_temp_c", 32.0f)
                put("os_version", "Android 14")
            })
        }
        sendMessage(json.toString())
    }

    private fun startHeartbeat() {
        pingJob?.cancel()
        pingJob = scope.launch {
            while (isActive) {
                delay(30_000)
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
        webSocket?.close(1000, "Client disconnect")
        webSocket = null
    }
}
