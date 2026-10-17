package com.anandacahya.kusuri.ui.clipboard

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
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
import com.anandacahya.kusuri.data.model.ClipboardItemDto
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.remote.KusuriWsEvent
import com.anandacahya.kusuri.data.repository.KusuriRepository
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ClipboardScreen(
    activeServer: ServerConfig?,
    repository: KusuriRepository
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val clipboardManager = remember {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }

    var clipboardItems by remember { mutableStateOf<List<ClipboardItemDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var showAddDialog by remember { mutableStateOf(false) }
    var newContentText by remember { mutableStateOf("") }

    fun loadClipboard() {
        if (activeServer == null) return
        scope.launch {
            isLoading = true
            errorMessage = null
            repository.getClipboard(activeServer).fold(
                onSuccess = {
                    clipboardItems = it
                    isLoading = false
                },
                onFailure = {
                    errorMessage = it.localizedMessage ?: "Gagal memuat clipboard"
                    isLoading = false
                }
            )
        }
    }

    LaunchedEffect(activeServer) {
        loadClipboard()
    }

    // Listen to real-time clipboard updates from WebSocket
    LaunchedEffect(repository.wsManager) {
        repository.wsManager.wsEvents.collectLatest { event ->
            if (event is KusuriWsEvent.ClipboardUpdated) {
                // Prepend or refresh
                clipboardItems = listOf(event.item) + clipboardItems.filter { it.id != event.item.id }
                Toast.makeText(context, "Clipboard baru dari ${event.item.source_device_name ?: "Perangkat Lain"}", Toast.LENGTH_SHORT).show()
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Shared Clipboard") },
                actions = {
                    // Quick button: Paste directly from device clipboard and upload
                    IconButton(onClick = {
                        val clip = clipboardManager.primaryClip
                        if (clip != null && clip.itemCount > 0) {
                            val text = clip.getItemAt(0).text?.toString() ?: ""
                            if (text.isNotBlank()) {
                                scope.launch {
                                    repository.createClipboard(activeServer, text).fold(
                                        onSuccess = {
                                            Toast.makeText(context, "Teks papan klip dibagikan!", Toast.LENGTH_SHORT).show()
                                            loadClipboard()
                                        },
                                        onFailure = {
                                            Toast.makeText(context, "Gagal membagikan: ${it.localizedMessage}", Toast.LENGTH_SHORT).show()
                                        }
                                    )
                                }
                            } else {
                                Toast.makeText(context, "Papan klip lokal kosong.", Toast.LENGTH_SHORT).show()
                            }
                        } else {
                            Toast.makeText(context, "Papan klip lokal kosong.", Toast.LENGTH_SHORT).show()
                        }
                    }) {
                        Icon(Icons.Default.ContentPasteGo, contentDescription = "Quick Paste & Share")
                    }
                    IconButton(onClick = { loadClipboard() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                }
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = {
                    newContentText = ""
                    showAddDialog = true
                },
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("Tulis Baru") }
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (isLoading && clipboardItems.isEmpty()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            } else if (clipboardItems.isEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ) {
                    Icon(
                        imageVector = Icons.Default.ContentPaste,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        "Papan Klip Bersama Kosong",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        "Salin teks dari komputer atau buat catatan baru di sini untuk disinkronkan ke seluruh perangkat Anda secara real-time.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
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

                    items(clipboardItems, key = { it.id }) { item ->
                        ClipboardItemCard(
                            item = item,
                            onCopy = {
                                val clip = ClipData.newPlainText("Kusuri Clipboard", item.content)
                                clipboardManager.setPrimaryClip(clip)
                                Toast.makeText(context, "Teks disalin ke papan klip HP!", Toast.LENGTH_SHORT).show()
                            },
                            onDelete = {
                                scope.launch {
                                    repository.deleteClipboard(activeServer, item.id)
                                    loadClipboard()
                                }
                            }
                        )
                    }
                }
            }

            // Dialog for adding manual text
            if (showAddDialog) {
                AlertDialog(
                    onDismissRequest = { showAddDialog = false },
                    title = { Text("Bagikan ke Papan Klip") },
                    text = {
                        Column {
                            OutlinedTextField(
                                value = newContentText,
                                onValueChange = { newContentText = it },
                                label = { Text("Teks / URL / Kode") },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .heightIn(min = 120.dp),
                                maxLines = 8
                            )
                        }
                    },
                    confirmButton = {
                        Button(
                            onClick = {
                                if (newContentText.isNotBlank()) {
                                    scope.launch {
                                        repository.createClipboard(activeServer, newContentText)
                                        showAddDialog = false
                                        loadClipboard()
                                    }
                                }
                            },
                            enabled = newContentText.isNotBlank()
                        ) {
                            Text("Bagikan")
                        }
                    },
                    dismissButton = {
                        TextButton(onClick = { showAddDialog = false }) {
                            Text("Batal")
                        }
                    }
                )
            }
        }
    }
}

@Composable
fun ClipboardItemCard(
    item: ClipboardItemDto,
    onCopy: () -> Unit,
    onDelete: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.Default.Devices,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.primary
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = item.source_device_name ?: "Cloud",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary
                    )
                }

                Text(
                    text = item.created_at.take(19).replace("T", " "),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            Spacer(modifier = Modifier.height(10.dp))

            Surface(
                color = MaterialTheme.colorScheme.surface,
                shape = MaterialTheme.shapes.small,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = item.content,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.padding(12.dp),
                    maxLines = 6,
                    overflow = TextOverflow.Ellipsis
                )
            }

            Spacer(modifier = Modifier.height(10.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End
            ) {
                TextButton(onClick = onCopy) {
                    Icon(Icons.Default.ContentCopy, contentDescription = null, modifier = Modifier.size(18.dp))
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("Salin ke HP")
                }
                Spacer(modifier = Modifier.width(8.dp))
                IconButton(onClick = onDelete) {
                    Icon(
                        Icons.Default.DeleteOutline,
                        contentDescription = "Hapus",
                        tint = MaterialTheme.colorScheme.error,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        }
    }
}
