package com.anandacahya.kusuri.data.repository

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.anandacahya.kusuri.data.model.ServerConfig
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.util.UUID

val Context.serverDataStore: DataStore<Preferences> by preferencesDataStore(name = "kusuri_servers")

class ServerRepository(private val context: Context) {

    private object PreferencesKeys {
        val SERVERS_JSON = stringPreferencesKey("servers_json")
        val ACTIVE_SERVER_ID = stringPreferencesKey("active_server_id")
    }

    private val json = Json { ignoreUnknownKeys = true }

    val serversFlow: Flow<List<ServerConfig>> = context.serverDataStore.data
        .map { preferences ->
            val jsonString = preferences[PreferencesKeys.SERVERS_JSON] ?: return@map defaultServers()
            try {
                json.decodeFromString<List<ServerConfig>>(jsonString)
            } catch (e: Exception) {
                defaultServers()
            }
        }

    val activeServerFlow: Flow<ServerConfig?> = context.serverDataStore.data
        .map { preferences ->
            val jsonString = preferences[PreferencesKeys.SERVERS_JSON] ?: return@map defaultServers().firstOrNull()
            val activeId = preferences[PreferencesKeys.ACTIVE_SERVER_ID]
            val servers = try {
                json.decodeFromString<List<ServerConfig>>(jsonString)
            } catch (e: Exception) {
                defaultServers()
            }
            servers.find { it.id == activeId } ?: servers.firstOrNull()
        }

    private fun defaultServers(): List<ServerConfig> {
        val defaultServer = ServerConfig(
            id = "default_local",
            name = "Localhost (Default)",
            host = "localhost",
            port = 8080,
            isHttps = false,
            isActive = true
        )
        return listOf(defaultServer)
    }

    suspend fun addServer(name: String, host: String, port: Int, isHttps: Boolean): ServerConfig {
        val newServer = ServerConfig(
            id = UUID.randomUUID().toString(),
            name = name,
            host = host,
            port = port,
            isHttps = isHttps,
            isActive = false
        )
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList + newServer
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
            if (updated.size == 1) {
                preferences[PreferencesKeys.ACTIVE_SERVER_ID] = newServer.id
            }
        }
        return newServer
    }

    suspend fun updateServer(server: ServerConfig) {
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList.map { if (it.id == server.id) server else it }
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
        }
    }

    suspend fun deleteServer(serverId: String) {
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList.filter { it.id != serverId }
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
            val activeId = preferences[PreferencesKeys.ACTIVE_SERVER_ID]
            if (activeId == serverId && updated.isNotEmpty()) {
                preferences[PreferencesKeys.ACTIVE_SERVER_ID] = updated.first().id
            }
        }
    }

    suspend fun setActiveServer(serverId: String) {
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList.map { it.copy(isActive = (it.id == serverId)) }
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
            preferences[PreferencesKeys.ACTIVE_SERVER_ID] = serverId
        }
    }

    suspend fun updateTokens(serverId: String, authToken: String?, deviceToken: String?) {
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList.map {
                if (it.id == serverId) {
                    it.copy(authToken = authToken ?: it.authToken, deviceToken = deviceToken ?: it.deviceToken)
                } else {
                    it
                }
            }
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
        }
    }

    suspend fun updatePairingInfo(
        serverId: String,
        deviceToken: String,
        deviceId: String,
        marketingName: String?,
        chipset: String?
    ) {
        context.serverDataStore.edit { preferences ->
            val currentList = getServers(preferences)
            val updated = currentList.map {
                if (it.id == serverId) {
                    it.copy(
                        deviceToken = deviceToken,
                        deviceId = deviceId,
                        marketingName = marketingName ?: it.marketingName,
                        chipset = chipset ?: it.chipset
                    )
                } else {
                    it
                }
            }
            preferences[PreferencesKeys.SERVERS_JSON] = json.encodeToString(updated)
        }
    }

    private fun getServers(preferences: Preferences): List<ServerConfig> {
        val jsonString = preferences[PreferencesKeys.SERVERS_JSON] ?: return defaultServers()
        return try {
            json.decodeFromString<List<ServerConfig>>(jsonString)
        } catch (e: Exception) {
            defaultServers()
        }
    }
}
