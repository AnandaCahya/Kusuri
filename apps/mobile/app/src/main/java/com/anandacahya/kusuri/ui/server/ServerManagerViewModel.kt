package com.anandacahya.kusuri.ui.server

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.anandacahya.kusuri.data.model.ServerConfig
import com.anandacahya.kusuri.data.remote.KusuriApiClientFactory
import com.anandacahya.kusuri.data.repository.ServerRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class ServerManagerViewModel(application: Application) : AndroidViewModel(application) {
    private val repository = ServerRepository(application)
    private val apiClientFactory = KusuriApiClientFactory()

    val servers: StateFlow<List<ServerConfig>> = repository.serversFlow
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    val activeServer: StateFlow<ServerConfig?> = repository.activeServerFlow
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), null)

    fun addServer(name: String, host: String, port: Int, isHttps: Boolean) {
        viewModelScope.launch {
            repository.addServer(name, host, port, isHttps)
        }
    }

    fun switchActiveServer(serverId: String) {
        viewModelScope.launch {
            repository.setActiveServer(serverId)
        }
    }

    fun deleteServer(serverId: String) {
        viewModelScope.launch {
            repository.deleteServer(serverId)
        }
    }

    suspend fun testConnection(server: ServerConfig): Boolean {
        return try {
            val service = apiClientFactory.createService(server)
            val health = service.healthCheck()
            health.status == "healthy"
        } catch (e: Exception) {
            false
        }
    }
}
