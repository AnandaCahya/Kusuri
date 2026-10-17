package com.anandacahya.kusuri.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class UserDto(
    val id: String,
    val username: String
)

@Serializable
data class AuthRequest(
    val username: String,
    val password: String
)

@Serializable
data class AuthResponse(
    val token: String,
    val user: UserDto
)

@Serializable
data class DeviceDto(
    val id: String,
    val user_id: String,
    val name: String,
    val device_type: String,
    val is_online: Boolean,
    val last_seen_at: String? = null,
    val created_at: String? = null,
    val raw_model: String? = null,
    val marketing_name: String? = null,
    val retail_branding: String? = null,
    val chipset: String? = null
)

@Serializable
data class UpdateDeviceRequest(
    val name: String
)

@Serializable
data class PairingCodeRequest(
    val device_name: String? = null,
    val device_type: String = "android"
)

@Serializable
data class PairingCodeResponse(
    val pairing_id: String,
    val code: String,
    val expires_at: String,
    val expires_in_seconds: Int,
    val max_attempts: Int
)

@Serializable
data class PairDeviceRequest(
    val pairing_code: String,
    val pairing_id: String? = null,
    val device_name: String? = null,
    val device_type: String = "android",
    val raw_model: String? = null
)

@Serializable
data class PairDeviceResponse(
    val device_id: String,
    val device_token: String,
    val auth_token: String? = null,
    val user_id: String,
    val device_name: String,
    val device_type: String,
    val marketing_name: String? = null,
    val chipset: String? = null
)

@Serializable
data class RawSpecsData(
    val sources: List<String>? = null,
    val source_urls: Map<String, String>? = null,
    val details: Map<String, String>? = null
)

@Serializable
data class GsmArenaSpecsDto(
    val id: String? = null,
    val device_name: String? = null,
    val marketing_name: String? = null,
    val brand: String? = null,
    val retail_branding: String? = null,
    val chipset: String? = null,
    val cpu: String? = null,
    val gpu: String? = null,
    val display_type: String? = null,
    val display_size: String? = null,
    val display_resolution: String? = null,
    val main_camera: String? = null,
    val camera_spec: String? = null,
    val selfie_camera: String? = null,
    val battery_type: String? = null,
    val battery_spec: String? = null,
    val battery_charging: String? = null,
    val charging_spec: String? = null,
    val memory_spec: String? = null,
    val os_original: String? = null,
    val dimensions: String? = null,
    val weight: String? = null,
    val picture_url: String? = null,
    val image_url: String? = null,
    val gsmarena_url: String? = null,
    val raw_specs: RawSpecsData? = null
)

@Serializable
data class LiveTelemetryDto(
    val imei1: String? = null,
    val imei2: String? = null,
    val serial_number: String? = null,
    val ram_total_mb: Long? = null,
    val ram_used_mb: Long? = null,
    val ram_free_mb: Long? = null,
    val storage_total_mb: Long? = null,
    val storage_used_mb: Long? = null,
    val storage_free_mb: Long? = null,
    val battery_percent: Float? = null,
    val battery_charging: Boolean? = null,
    val battery_temp_c: Float? = null,
    val battery_health: String? = null,
    val os_version: String? = null,
    val android_sdk: Int? = null,
    val kernel_version: String? = null,
    val screen_resolution: String? = null,
    val screen_refresh_rate: Float? = null,
    val ip_address: String? = null,
    val network_type: String? = null,
    val cpu_model: String? = null,
    val cpu_cores: Int? = null,
    val cpu_logical_cores: Int? = null,
    val cpu_frequency_mhz: Long? = null,
    val gpu_model: String? = null,
    val hostname: String? = null,
    val architecture: String? = null,
    val uptime_formatted: String? = null,
    val disks_info: String? = null,
    val updated_at: String? = null
)

@Serializable
data class EnrichedDeviceSpecsResponse(
    val device_id: String? = null,
    val name: String? = null,
    val device_type: String? = null,
    val raw_model: String? = null,
    val marketing_name: String? = null,
    val retail_branding: String? = null,
    val chipset: String? = null,
    val is_online: Boolean? = null,
    val catalog_specs: GsmArenaSpecsDto? = null,
    val live_hardware: LiveTelemetryDto? = null
)

@Serializable
data class ClipboardItemDto(
    val id: String,
    val user_id: String? = null,
    val source_device_id: String? = null,
    val source_device_name: String? = null,
    val content: String,
    val content_type: String = "text/plain",
    val created_at: String
)

@Serializable
data class CreateClipboardRequest(
    val content: String,
    val content_type: String = "text/plain"
)

@Serializable
data class SharedFileDto(
    val id: String,
    val user_id: String? = null,
    val source_device_id: String? = null,
    val source_device_name: String? = null,
    val target_device_id: String? = null,
    val original_filename: String,
    val file_size: Long,
    val mime_type: String,
    val download_url: String? = null,
    val created_at: String
)

@Serializable
data class RemoteCommandDto(
    val id: String,
    val user_id: String? = null,
    val source_device_id: String? = null,
    val source_device_name: String? = null,
    val target_device_id: String,
    val target_device_name: String? = null,
    val command: String,
    val payload: JsonElement? = null,
    val status: String,
    val response: JsonElement? = null,
    val created_at: String,
    val updated_at: String? = null
)

@Serializable
data class SendCommandRequest(
    val target_device_id: String,
    val command: String,
    val payload: JsonElement? = null
)

@Serializable
data class CommandResultRequest(
    val command_id: String,
    val status: String,
    val response: JsonElement? = null
)

@Serializable
data class SystemHealthDto(
    val status: String,
    val service: String? = null,
    val version: String? = null,
    val timestamp: String? = null
)

@Serializable
data class HardwareTelemetryRequest(
    val raw_model: String? = null,
    val imei1: String? = null,
    val imei2: String? = null,
    val serial_number: String? = null,
    val ram_total_mb: Long,
    val ram_used_mb: Long,
    val ram_free_mb: Long? = null,
    val storage_total_mb: Long,
    val storage_used_mb: Long,
    val storage_free_mb: Long? = null,
    val battery_percent: Float,
    val battery_charging: Boolean,
    val battery_temp_c: Float,
    val battery_health: String? = null,
    val os_version: String,
    val android_sdk: Int,
    val kernel_version: String? = null,
    val screen_resolution: String? = null,
    val screen_refresh_rate: Float? = null,
    val ip_address: String? = null,
    val network_type: String? = null
)

@Serializable
data class IngestNotificationRequest(
    val device_id: String? = null,
    val package_or_app_name: String,
    val app_title: String? = null,
    val title: String,
    val content: String,
    val subtext: String? = null,
    val posted_at: String? = null
)

@Serializable
data class NotificationResponse(
    val id: String,
    val user_id: String? = null,
    val device_id: String? = null,
    val device_name: String? = null,
    val package_or_app_name: String,
    val app_title: String? = null,
    val title: String,
    val content: String,
    val subtext: String? = null,
    val posted_at: String,
    val created_at: String? = null
)

@Serializable
data class GenericMessageResponse(
    val success: Boolean? = null,
    val message: String? = null
)
