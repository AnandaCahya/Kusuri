package com.anandacahya.kusuri.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ServerConfig(
    val id: String,
    val name: String,           // e.g. "Home Server", "Office Server"
    val host: String,           // e.g. "192.168.1.100" or "kusuri.mydns.org"
    val port: Int = 8080,       // e.g. 8080
    val isHttps: Boolean = false,
    val authToken: String? = null,    // User JWT token
    val deviceToken: String? = null,  // Device permanent pairing token (ksr_...)
    val deviceId: String? = null,     // Paired Device UUID
    val marketingName: String? = null,// Resolved marketing name from backend (e.g., "Infinix HOT 50 Pro")
    val chipset: String? = null,      // Resolved chipset from GSM Arena (e.g., "Mediatek Helio G100")
    val isActive: Boolean = false
) {
    val baseUrl: String
        get() {
            val scheme = if (isHttps) "https" else "http"
            return "$scheme://$host:$port"
        }
    
    val wsUrl: String
        get() {
            val scheme = if (isHttps) "wss" else "ws"
            return "$scheme://$host:$port/ws"
        }
}
