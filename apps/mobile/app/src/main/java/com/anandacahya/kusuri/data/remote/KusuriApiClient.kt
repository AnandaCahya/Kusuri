package com.anandacahya.kusuri.data.remote

import com.anandacahya.kusuri.data.model.*
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.MultipartBody
import okhttp3.OkHttpClient
import okhttp3.RequestBody
import okhttp3.ResponseBody
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.kotlinx.serialization.asConverterFactory
import retrofit2.http.*
import java.util.concurrent.TimeUnit

interface KusuriApiService {
    @GET("api/health")
    suspend fun healthCheck(): SystemHealthDto

    // Auth
    @POST("api/auth/register")
    suspend fun register(@Body request: AuthRequest): AuthResponse

    @POST("api/auth/login")
    suspend fun login(@Body request: AuthRequest): AuthResponse

    @GET("api/auth/me")
    suspend fun getMe(@Header("Authorization") authHeader: String? = null): UserDto

    // Devices
    @POST("api/devices/pairing-code")
    suspend fun requestPairingCode(
        @Header("Authorization") authHeader: String? = null,
        @Body request: PairingCodeRequest = PairingCodeRequest()
    ): PairingCodeResponse

    @POST("api/devices/pair")
    suspend fun pairDevice(@Body request: PairDeviceRequest): PairDeviceResponse

    @GET("api/devices")
    suspend fun getDevices(@Header("Authorization") authHeader: String? = null): List<DeviceDto>

    @GET("api/devices/{id}/specs")
    suspend fun getDeviceSpecs(
        @Path("id") deviceId: String,
        @Header("Authorization") authHeader: String? = null
    ): EnrichedDeviceSpecsResponse

    @PATCH("api/devices/{id}")
    suspend fun updateDevice(
        @Path("id") deviceId: String,
        @Body request: UpdateDeviceRequest,
        @Header("Authorization") authHeader: String? = null
    ): Response<Unit>

    @DELETE("api/devices/{id}")
    suspend fun deleteDevice(
        @Path("id") deviceId: String,
        @Header("Authorization") authHeader: String? = null
    ): Response<Unit>

    @POST("api/devices/sync-catalog")
    suspend fun syncCatalog(
        @Header("Authorization") authHeader: String? = null
    ): GenericMessageResponse

    @POST("api/devices/{id}/telemetry")
    suspend fun sendHardwareTelemetry(
        @Path("id") deviceId: String,
        @Body request: HardwareTelemetryRequest,
        @Header("Authorization") authHeader: String? = null
    ): EnrichedDeviceSpecsResponse

    // Clipboard
    @GET("api/clipboard")
    suspend fun getClipboard(
        @Header("Authorization") authHeader: String? = null
    ): List<ClipboardItemDto>

    @POST("api/clipboard")
    suspend fun postClipboard(
        @Body request: CreateClipboardRequest,
        @Header("Authorization") authHeader: String? = null
    ): ClipboardItemDto

    @DELETE("api/clipboard/{id}")
    suspend fun deleteClipboard(
        @Path("id") id: String,
        @Header("Authorization") authHeader: String? = null
    ): Response<Unit>

    // Files
    @GET("api/files")
    suspend fun getFiles(
        @Header("Authorization") authHeader: String? = null
    ): List<SharedFileDto>

    @Multipart
    @POST("api/files/upload")
    suspend fun uploadFile(
        @Part file: MultipartBody.Part,
        @Part("target_device_id") targetDeviceId: RequestBody? = null,
        @Header("Authorization") authHeader: String? = null
    ): SharedFileDto

    @DELETE("api/files/{id}")
    suspend fun deleteFile(
        @Path("id") id: String,
        @Header("Authorization") authHeader: String? = null
    ): Response<Unit>

    @Streaming
    @GET("api/files/{id}/download")
    suspend fun downloadFile(
        @Path("id") id: String,
        @Header("Authorization") authHeader: String? = null
    ): ResponseBody

    // Commands
    @GET("api/commands")
    suspend fun getCommands(
        @Header("Authorization") authHeader: String? = null
    ): List<RemoteCommandDto>

    @POST("api/commands")
    suspend fun sendCommand(
        @Body request: SendCommandRequest,
        @Header("Authorization") authHeader: String? = null
    ): RemoteCommandDto

    @POST("api/commands/{id}/result")
    suspend fun reportCommandResult(
        @Path("id") commandId: String,
        @Body request: CommandResultRequest,
        @Header("Authorization") authHeader: String? = null
    ): GenericMessageResponse

    // Notifications
    @POST("api/notifications")
    suspend fun postNotification(
        @Body request: IngestNotificationRequest,
        @Header("Authorization") authHeader: String? = null
    ): NotificationResponse

    @GET("api/notifications")
    suspend fun getNotifications(
        @Query("device_id") deviceId: String? = null,
        @Query("search") search: String? = null,
        @Query("limit") limit: Int? = null,
        @Header("Authorization") authHeader: String? = null
    ): List<NotificationResponse>
}

class KusuriApiClientFactory {
    private val json = Json { 
        ignoreUnknownKeys = true 
        isLenient = true 
        encodeDefaults = true
    }

    fun createService(serverConfig: ServerConfig): KusuriApiService {
        val logging = HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BASIC
        }

        val authInterceptor = Interceptor { chain ->
            val original = chain.request()
            val requestBuilder = original.newBuilder()
            
            // Prefer user authToken if available, fallback to deviceToken
            val token = serverConfig.authToken ?: serverConfig.deviceToken
            if (!token.isNullOrBlank() && original.header("Authorization") == null) {
                requestBuilder.header("Authorization", "Bearer $token")
            }
            if (!serverConfig.deviceToken.isNullOrBlank() && original.header("X-Device-Token") == null) {
                requestBuilder.header("X-Device-Token", serverConfig.deviceToken)
            }
            chain.proceed(requestBuilder.build())
        }

        val client = OkHttpClient.Builder()
            .addInterceptor(authInterceptor)
            .addInterceptor(logging)
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .build()

        val baseUrlNormalized = if (serverConfig.baseUrl.endsWith("/")) serverConfig.baseUrl else "${serverConfig.baseUrl}/"

        val retrofit = Retrofit.Builder()
            .baseUrl(baseUrlNormalized)
            .client(client)
            .addConverterFactory(json.asConverterFactory("application/json".toMediaType()))
            .build()

        return retrofit.create(KusuriApiService::class.java)
    }
}
