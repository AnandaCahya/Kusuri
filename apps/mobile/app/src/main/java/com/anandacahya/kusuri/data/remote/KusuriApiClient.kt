package com.anandacahya.kusuri.data.remote

import com.anandacahya.kusuri.data.model.ServerConfig
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.kotlinx.serialization.asConverterFactory
import retrofit2.http.*

@Serializable
data class AuthRequest(val username: String, val password: String)

@Serializable
data class AuthResponse(val token: String, val user: UserDto)

@Serializable
data class UserDto(val id: String, val username: String)

@Serializable
data class PairingCodeRequest(val device_name: String? = null, val device_type: String = "android")

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
    val device_name: String,
    val device_type: String = "android",
    val raw_model: String
)

@Serializable
data class PairDeviceResponse(
    val device_id: String,
    val device_token: String,
    val user_id: String,
    val device_name: String,
    val device_type: String,
    val marketing_name: String?,
    val chipset: String?
)

@Serializable
data class ClipboardItemDto(
    val id: String,
    val content: String,
    val content_type: String = "text/plain",
    val source_device_name: String?,
    val created_at: String
)

@Serializable
data class CreateClipboardRequest(
    val content: String,
    val content_type: String = "text/plain"
)

@Serializable
data class HardwareTelemetryRequest(
    val raw_model: String,
    val imei1: String? = null,
    val imei2: String? = null,
    val ram_total_mb: Long,
    val ram_used_mb: Long,
    val storage_total_mb: Long,
    val storage_used_mb: Long,
    val battery_percent: Float,
    val battery_charging: Boolean,
    val battery_temp_c: Float,
    val os_version: String,
    val android_sdk: Int
)

interface KusuriApiService {
    @GET("api/health")
    suspend fun healthCheck(): Map<String, String>

    @POST("api/auth/register")
    suspend fun register(@Body request: AuthRequest): AuthResponse

    @POST("api/auth/login")
    suspend fun login(@Body request: AuthRequest): AuthResponse

    @POST("api/devices/pairing-code")
    suspend fun requestPairingCode(
        @Header("Authorization") authHeader: String,
        @Body request: PairingCodeRequest
    ): PairingCodeResponse

    @POST("api/devices/pair")
    suspend fun pairDevice(@Body request: PairDeviceRequest): PairDeviceResponse

    @GET("api/devices")
    suspend fun getDevices(@Header("Authorization") authHeader: String): List<PairDeviceResponse>

    @GET("api/clipboard")
    suspend fun getClipboard(@Header("Authorization") authHeader: String): List<ClipboardItemDto>

    @POST("api/clipboard")
    suspend fun postClipboard(
        @Header("Authorization") authHeader: String,
        @Body request: CreateClipboardRequest
    ): ClipboardItemDto

    @POST("api/devices/{id}/telemetry")
    suspend fun sendHardwareTelemetry(
        @Header("Authorization") authHeader: String,
        @Path("id") deviceId: String,
        @Body request: HardwareTelemetryRequest
    ): kotlinx.serialization.json.JsonObject
}

class KusuriApiClientFactory {
    private val json = Json { ignoreUnknownKeys = true; isLenient = true }

    fun createService(serverConfig: ServerConfig): KusuriApiService {
        val logging = HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BODY
        }

        val authInterceptor = Interceptor { chain ->
            val original = chain.request()
            val requestBuilder = original.newBuilder()
            
            val token = serverConfig.deviceToken ?: serverConfig.authToken
            if (!token.isNullOrBlank()) {
                requestBuilder.header("Authorization", "Bearer $token")
            }
            chain.proceed(requestBuilder.build())
        }

        val client = OkHttpClient.Builder()
            .addInterceptor(authInterceptor)
            .addInterceptor(logging)
            .build()

        val retrofit = Retrofit.Builder()
            .baseUrl(serverConfig.baseUrl.let { if (it.endsWith("/")) it else "$it/" })
            .client(client)
            .addConverterFactory(json.asConverterFactory("application/json".toMediaType()))
            .build()

        return retrofit.create(KusuriApiService::class.java)
    }
}
