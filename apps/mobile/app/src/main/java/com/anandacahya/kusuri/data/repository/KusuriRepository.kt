package com.anandacahya.kusuri.data.repository

import android.content.Context
import android.net.Uri
import android.os.Environment
import android.provider.OpenableColumns
import android.util.Log
import com.anandacahya.kusuri.data.model.*
import com.anandacahya.kusuri.data.remote.KusuriApiClientFactory
import com.anandacahya.kusuri.data.remote.KusuriApiService
import com.anandacahya.kusuri.data.remote.KusuriWebSocketManager
import com.anandacahya.kusuri.data.telemetry.DeviceTelemetryCollector
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.asRequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.File
import java.io.FileOutputStream

class KusuriRepository(
    private val context: Context,
    val serverRepository: ServerRepository,
    val scope: CoroutineScope
) {
    private val TAG = "KusuriRepo"
    private val apiFactory = KusuriApiClientFactory()
    val wsManager = KusuriWebSocketManager(context, scope)
    val telemetryCollector = DeviceTelemetryCollector(context)
    private val json = Json { ignoreUnknownKeys = true }

    val activeServerFlow: Flow<ServerConfig?> = serverRepository.activeServerFlow

    private fun getService(server: ServerConfig?): KusuriApiService {
        val cfg = server ?: throw IllegalStateException("Tidak ada server Kusuri yang aktif. Silakan pilih server di Settings.")
        return apiFactory.createService(cfg)
    }

    suspend fun getHealth(server: ServerConfig?): Result<SystemHealthDto> = withContext(Dispatchers.IO) {
        runCatching { getService(server).healthCheck() }
    }

    suspend fun login(server: ServerConfig?, req: AuthRequest): Result<AuthResponse> = withContext(Dispatchers.IO) {
        runCatching {
            val res = getService(server).login(req)
            if (server != null && res.token.isNotBlank()) {
                serverRepository.updateTokens(server.id, authToken = res.token, deviceToken = null)
            }
            res
        }
    }

    suspend fun register(server: ServerConfig?, req: AuthRequest): Result<AuthResponse> = withContext(Dispatchers.IO) {
        runCatching {
            val res = getService(server).register(req)
            if (server != null && res.token.isNotBlank()) {
                serverRepository.updateTokens(server.id, authToken = res.token, deviceToken = null)
            }
            res
        }
    }

    suspend fun getDevices(server: ServerConfig?): Result<List<DeviceDto>> = withContext(Dispatchers.IO) {
        runCatching { getService(server).getDevices() }
    }

    suspend fun getDeviceSpecs(server: ServerConfig?, deviceId: String): Result<EnrichedDeviceSpecsResponse> = withContext(Dispatchers.IO) {
        runCatching { getService(server).getDeviceSpecs(deviceId) }
    }

    suspend fun deleteDevice(server: ServerConfig?, deviceId: String): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            getService(server).deleteDevice(deviceId)
            Unit
        }
    }

    suspend fun createPairingCode(server: ServerConfig?): Result<PairingCodeResponse> = withContext(Dispatchers.IO) {
        runCatching { getService(server).requestPairingCode() }
    }

    suspend fun syncGoogleCatalog(server: ServerConfig?): Result<GenericMessageResponse> = withContext(Dispatchers.IO) {
        runCatching { getService(server).syncCatalog() }
    }

    suspend fun syncHardwareTelemetry(server: ServerConfig?): Result<EnrichedDeviceSpecsResponse> = withContext(Dispatchers.IO) {
        runCatching {
            val deviceId = server?.deviceId ?: throw IllegalStateException("Perangkat ini belum di-pair (tidak memiliki Device ID).")
            val req = telemetryCollector.toTelemetryRequest()
            getService(server).sendHardwareTelemetry(deviceId, req)
        }
    }

    suspend fun getClipboard(server: ServerConfig?): Result<List<ClipboardItemDto>> = withContext(Dispatchers.IO) {
        runCatching { getService(server).getClipboard() }
    }

    suspend fun createClipboard(server: ServerConfig?, content: String): Result<ClipboardItemDto> = withContext(Dispatchers.IO) {
        runCatching {
            getService(server).postClipboard(CreateClipboardRequest(content = content))
        }
    }

    suspend fun deleteClipboard(server: ServerConfig?, id: String): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            getService(server).deleteClipboard(id)
            Unit
        }
    }

    suspend fun getFiles(server: ServerConfig?): Result<List<SharedFileDto>> = withContext(Dispatchers.IO) {
        runCatching { getService(server).getFiles() }
    }

    suspend fun uploadFile(
        server: ServerConfig?,
        fileUri: Uri,
        targetDeviceId: String? = null
    ): Result<SharedFileDto> = withContext(Dispatchers.IO) {
        runCatching {
            val contentResolver = context.contentResolver
            var fileName = "upload_${System.currentTimeMillis()}"
            var fileSize = 0L

            contentResolver.query(fileUri, null, null, null, null)?.use { cursor ->
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (cursor.moveToFirst()) {
                    if (nameIndex != -1) fileName = cursor.getString(nameIndex)
                    if (sizeIndex != -1) fileSize = cursor.getLong(sizeIndex)
                }
            }

            val mimeType = contentResolver.getType(fileUri) ?: "application/octet-stream"

            // Copy to temp cache file
            val tempFile = File(context.cacheDir, fileName)
            contentResolver.openInputStream(fileUri)?.use { input ->
                FileOutputStream(tempFile).use { output ->
                    input.copyTo(output)
                }
            } ?: throw IllegalStateException("Gagal membaca file dari penyimpanan.")

            val requestFile = tempFile.asRequestBody(mimeType.toMediaTypeOrNull())
            val filePart = MultipartBody.Part.createFormData("file", fileName, requestFile)
            val targetPart = targetDeviceId?.toRequestBody("text/plain".toMediaTypeOrNull())

            val res = getService(server).uploadFile(file = filePart, targetDeviceId = targetPart)
            tempFile.delete()
            res
        }
    }

    suspend fun downloadFileToStorage(
        server: ServerConfig?,
        fileId: String,
        originalFilename: String
    ): Result<File> = withContext(Dispatchers.IO) {
        runCatching {
            val responseBody = getService(server).downloadFile(fileId)
            val downloadsDir = context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)
                ?: context.filesDir
            downloadsDir.mkdirs()
            val targetFile = File(downloadsDir, originalFilename)

            responseBody.byteStream().use { input ->
                FileOutputStream(targetFile).use { output ->
                    input.copyTo(output)
                }
            }
            targetFile
        }
    }

    suspend fun deleteFile(server: ServerConfig?, id: String): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            getService(server).deleteFile(id)
            Unit
        }
    }

    suspend fun getCommands(server: ServerConfig?): Result<List<RemoteCommandDto>> = withContext(Dispatchers.IO) {
        runCatching { getService(server).getCommands() }
    }

    suspend fun sendCommand(
        server: ServerConfig?,
        targetDeviceId: String,
        command: String,
        payload: JsonElement? = null
    ): Result<RemoteCommandDto> = withContext(Dispatchers.IO) {
        runCatching {
            getService(server).sendCommand(
                SendCommandRequest(
                    target_device_id = targetDeviceId,
                    command = command,
                    payload = payload
                )
            )
        }
    }

    companion object {
        @Volatile
        private var instance: KusuriRepository? = null

        fun getInstance(context: Context): KusuriRepository {
            return instance ?: synchronized(this) {
                instance ?: run {
                    val appCtx = context.applicationContext
                    val serverRepo = ServerRepository(appCtx)
                    val appScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.SupervisorJob() + Dispatchers.Default)
                    KusuriRepository(appCtx, serverRepo, appScope).also {
                        instance = it
                    }
                }
            }
        }
    }
}
