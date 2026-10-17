package com.anandacahya.kusuri.service

import android.app.Notification
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log
import com.anandacahya.kusuri.data.model.IngestNotificationRequest
import com.anandacahya.kusuri.data.remote.KusuriApiClientFactory
import com.anandacahya.kusuri.data.repository.ServerRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/**
 * 24/7 Android Notification Listener Service for Kusuri
 * Reads incoming system & app notifications and seamlessly streams them to the Kusuri backend.
 */
class KusuriNotificationListenerService : NotificationListenerService() {

    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val apiFactory = KusuriApiClientFactory()
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US).apply {
        timeZone = TimeZone.getTimeZone("UTC")
    }

    override fun onListenerConnected() {
        super.onListenerConnected()
        Log.i(TAG, "KusuriNotificationListenerService terhubung ke Android Notification Center")
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        super.onNotificationPosted(sbn)
        if (sbn == null) return

        val packageName = sbn.packageName ?: return
        // Abaikan notifikasi dari aplikasi Kusuri sendiri agar tidak recursive
        if (packageName == applicationContext.packageName) return

        val notification = sbn.notification ?: return
        val extras = notification.extras ?: return

        // Ekstraksi judul dan isi pesan notifikasi
        val title = extras.getCharSequence(Notification.EXTRA_TITLE)?.toString()?.trim() ?: ""
        val text = extras.getCharSequence(Notification.EXTRA_TEXT)?.toString()?.trim() ?: ""
        val bigText = extras.getCharSequence(Notification.EXTRA_BIG_TEXT)?.toString()?.trim()
        val subText = extras.getCharSequence(Notification.EXTRA_SUB_TEXT)?.toString()?.trim()

        val content = if (!bigText.isNullOrBlank()) bigText else text

        // Jika tidak ada teks maupun judul, lewati
        if (title.isBlank() && content.isBlank()) return

        // Dapatkan nama aplikasi pengguna
        val pm = applicationContext.packageManager
        val appTitle = try {
            val appInfo = pm.getApplicationInfo(packageName, 0)
            pm.getApplicationLabel(appInfo).toString()
        } catch (e: Exception) {
            packageName
        }

        val postedAtIso = dateFormat.format(Date(sbn.postTime))

        // Kirimkan ke Kusuri Backend secara asynchronous
        serviceScope.launch {
            try {
                val serverRepo = ServerRepository(applicationContext)
                val activeServer = serverRepo.getActiveServer() ?: return@launch

                val api = apiFactory.createService(activeServer)
                val req = IngestNotificationRequest(
                    device_id = activeServer.deviceId,
                    package_or_app_name = packageName,
                    app_title = appTitle,
                    title = title.ifBlank { appTitle },
                    content = content,
                    subtext = subText,
                    posted_at = postedAtIso
                )

                api.postNotification(req)
                Log.d(TAG, "Notifikasi dari '$appTitle' ($packageName) sukses dikirim ke Kusuri Backend")
            } catch (e: Exception) {
                Log.w(TAG, "Gagal mengirim notifikasi ke Kusuri Backend: ${e.message}")
            }
        }
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        super.onNotificationRemoved(sbn)
    }

    companion object {
        private const val TAG = "KusuriNotificationListener"
    }
}
