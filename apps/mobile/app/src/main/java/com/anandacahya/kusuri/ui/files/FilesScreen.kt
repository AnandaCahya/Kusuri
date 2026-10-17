package com.anandacahya.kusuri.ui.files

import android.content.Context
import android.net.Uri
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.anandacahya.kusuri.data.model.DeviceDto
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.model.SharedFileDto
import com.anandacahya.kusuri.data.remote.KusuriWsEvent
import com.anandacahya.kusuri.data.repository.KusuriRepository
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FilesScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var files by remember { mutableStateOf<List<SharedFileDto>>(emptyList()) }
    var onlineDevices by remember { mutableStateOf<List<DeviceDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var isUploading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var selectedTargetDeviceId by remember { mutableStateOf<String?>(null) }
    var showTargetDevicePicker by remember { mutableStateOf(false) }
    var pendingUploadUri by remember { mutableStateOf<Uri?>(null) }

    fun loadFiles() {
        if (activeServer == null) return
        scope.launch {
            isLoading = true
            errorMessage = null
            repository.getFiles(activeServer).fold(
                onSuccess = {
                    files = it
                    isLoading = false
                },
                onFailure = {
                    errorMessage = it.localizedMessage ?: "Gagal memuat daftar berkas"
                    isLoading = false
                }
            )
        }
    }

    fun loadDevices() {
        if (activeServer == null) return
        scope.launch {
            repository.getDevices(activeServer).fold(
                onSuccess = { onlineDevices = it },
                onFailure = { /* ignore */ }
            )
        }
    }

    LaunchedEffect(activeServer) {
        loadFiles()
        loadDevices()
    }

    // Real-time listener for incoming files
    LaunchedEffect(repository.wsManager) {
        repository.wsManager.wsEvents.collectLatest { event ->
            if (event is KusuriWsEvent.FileAvailable) {
                Toast.makeText(context, "Berkas baru tersedia: ${event.originalFilename}", Toast.LENGTH_SHORT).show()
                loadFiles()
            }
        }
    }

    // System File Picker Launcher
    val filePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri: Uri? ->
        if (uri != null) {
            pendingUploadUri = uri
            showTargetDevicePicker = true
        }
    }

    fun startUpload(uri: Uri, targetDeviceId: String?) {
        scope.launch {
            isUploading = true
            repository.uploadFile(activeServer, uri, targetDeviceId).fold(
                onSuccess = {
                    isUploading = false
                    Toast.makeText(context, "Berkas '${it.original_filename}' berhasil diunggah!", Toast.LENGTH_LONG).show()
                    loadFiles()
                },
                onFailure = {
                    isUploading = false
                    Toast.makeText(context, "Gagal mengunggah berkas: ${it.localizedMessage}", Toast.LENGTH_LONG).show()
                }
            )
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Kusuri Drop (Shared Files)") },
                actions = {
                    IconButton(onClick = { loadFiles() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                }
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { filePickerLauncher.launch("*/*") },
                icon = { Icon(Icons.Default.UploadFile, contentDescription = null) },
                text = { Text("Kirim Berkas") }
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (isUploading) {
                LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            }

            if (isLoading && files.isEmpty()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            } else if (files.isEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ) {
                    Icon(
                        imageVector = Icons.Default.FolderShared,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        "Belum Ada Berkas Dibagikan",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        "Kirim berkas dari HP ini ke PC atau terima berkas dari laptop / browser tanpa batas kecepatan lokal.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Button(onClick = { filePickerLauncher.launch("*/*") }) {
                        Icon(Icons.Default.Upload, contentDescription = null)
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Pilih & Kirim Berkas")
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    if (errorMessage != null) {
                        item {
                            Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer)) {
                                Text(
                                    text = errorMessage ?: "",
                                    modifier = Modifier.padding(12.dp),
                                    color = MaterialTheme.colorScheme.onErrorContainer
                                )
                            }
                        }
                    }

                    items(files, key = { it.id }) { file ->
                        FileItemCard(
                            file = file,
                            onDownload = {
                                scope.launch {
                                    Toast.makeText(context, "Mengunduh ${file.original_filename}...", Toast.LENGTH_SHORT).show()
                                    repository.downloadFileToStorage(activeServer, file.id, file.original_filename).fold(
                                        onSuccess = { savedFile ->
                                            Toast.makeText(context, "Berkas tersimpan di: ${savedFile.absolutePath}", Toast.LENGTH_LONG).show()
                                        },
                                        onFailure = {
                                            Toast.makeText(context, "Gagal mengunduh: ${it.localizedMessage}", Toast.LENGTH_LONG).show()
                                        }
                                    )
                                }
                            },
                            onDelete = {
                                scope.launch {
                                    repository.deleteFile(activeServer, file.id)
                                    loadFiles()
                                }
                            }
                        )
                    }
                }
            }

            // Target Device Picker Dialog when uploading
            if (showTargetDevicePicker && pendingUploadUri != null) {
                AlertDialog(
                    onDismissRequest = {
                        showTargetDevicePicker = false
                        pendingUploadUri = null
                    },
                    title = { Text("Kirim Berkas Ke:") },
                    text = {
                        Column {
                            Text(
                                "Pilih perangkat tujuan spesifik (opsional) atau kirim ke seluruh perangkat Anda:",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Spacer(modifier = Modifier.height(12.dp))

                            TextButton(
                                onClick = {
                                    val uri = pendingUploadUri
                                    showTargetDevicePicker = false
                                    pendingUploadUri = null
                                    if (uri != null) startUpload(uri, null)
                                },
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Icon(Icons.Default.Cloud, contentDescription = null)
                                    Spacer(modifier = Modifier.width(12.dp))
                                    Text("Semua Perangkat (Shared Pool)")
                                }
                            }

                            onlineDevices.forEach { dev ->
                                TextButton(
                                    onClick = {
                                        val uri = pendingUploadUri
                                        showTargetDevicePicker = false
                                        pendingUploadUri = null
                                        if (uri != null) startUpload(uri, dev.id)
                                    },
                                    modifier = Modifier.fillMaxWidth()
                                ) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Icon(Icons.Default.Smartphone, contentDescription = null)
                                        Spacer(modifier = Modifier.width(12.dp))
                                        Text(dev.marketing_name ?: dev.name)
                                    }
                                }
                            }
                        }
                    },
                    confirmButton = {},
                    dismissButton = {
                        TextButton(onClick = {
                            showTargetDevicePicker = false
                            pendingUploadUri = null
                        }) {
                            Text("Batal")
                        }
                    }
                )
            }
        }
    }
}

@Composable
fun FileItemCard(
    file: SharedFileDto,
    onDownload: () -> Unit,
    onDelete: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val fileIcon = when {
                file.mime_type.startsWith("image/") -> Icons.Default.Image
                file.mime_type.startsWith("video/") -> Icons.Default.VideoFile
                file.mime_type.startsWith("audio/") -> Icons.Default.AudioFile
                file.mime_type.contains("pdf") -> Icons.Default.PictureAsPdf
                file.mime_type.contains("zip") || file.mime_type.contains("compressed") -> Icons.Default.FolderZip
                else -> Icons.Default.InsertDriveFile
            }

            Icon(
                imageVector = fileIcon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(36.dp)
            )

            Spacer(modifier = Modifier.width(14.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = file.original_filename,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "${formatFileSize(file.file_size)} • Dari ${file.source_device_name ?: "Cloud"}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            IconButton(onClick = onDownload) {
                Icon(Icons.Default.Download, contentDescription = "Unduh", tint = MaterialTheme.colorScheme.primary)
            }

            IconButton(onClick = onDelete) {
                Icon(Icons.Default.DeleteOutline, contentDescription = "Hapus", tint = MaterialTheme.colorScheme.error)
            }
        }
    }
}

private fun formatFileSize(bytes: Long): String {
    if (bytes < 1024) return "$bytes B"
    val kb = bytes / 1024.0
    if (kb < 1024) return String.format(Locale.US, "%.1f KB", kb)
    val mb = kb / 1024.0
    if (mb < 1024) return String.format(Locale.US, "%.1f MB", mb)
    val gb = mb / 1024.0
    return String.format(Locale.US, "%.2f GB", gb)
}
