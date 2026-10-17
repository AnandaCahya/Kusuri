package com.anandacahya.kusuri.data.telemetry

import android.app.ActivityManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.BatteryManager
import android.os.Build
import android.os.Environment
import android.os.StatFs
import com.anandacahya.kusuri.data.model.HardwareTelemetryRequest
import java.net.Inet4Address
import java.net.NetworkInterface

data class DeviceInfoState(
    val rawModel: String,
    val brand: String,
    val product: String,
    val marketingName: String? = null,
    val displayName: String,
    val androidVersion: String,
    val androidSdk: Int,
    val kernelVersion: String? = null,
    val socHardware: String? = null,
    val ramTotalMb: Long,
    val ramUsedMb: Long,
    val ramFreeMb: Long,
    val storageTotalMb: Long,
    val storageUsedMb: Long,
    val storageFreeMb: Long,
    val batteryPercent: Float,
    val batteryCharging: Boolean,
    val batteryTempC: Float,
    val batteryHealth: String = "Good",
    val screenResolution: String = "",
    val screenRefreshRate: Float = 60f,
    val ipAddress: String? = null,
    val networkType: String? = null
) {
    val sdkInt: Int get() = androidSdk
}

class DeviceTelemetryCollector(private val context: Context) {

    // Cached static specifications to optimize battery & CPU usage
    private val brandInfo by lazy { BrandDeviceResolver.resolve(context) }
    private val androidVersion = "Android ${Build.VERSION.RELEASE}"
    private val androidSdk = Build.VERSION.SDK_INT
    private val product = Build.PRODUCT ?: "Unknown"

    private val screenRes: String by lazy {
        val dm = context.resources.displayMetrics
        "${dm.widthPixels} x ${dm.heightPixels}"
    }

    private val screenRefresh: Float by lazy {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                context.display?.refreshRate ?: 60.0f
            } else {
                60.0f
            }
        } catch (_: Exception) {
            60.0f
        }
    }

    // Static hardware limits (RAM total and storage total rarely change)
    private var cachedRamTotalMb: Long = 0L
    private var cachedStorageTotalMb: Long = 0L

    fun collectDeviceInfo(): DeviceInfoState {
        // RAM Info
        val actManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val memInfo = ActivityManager.MemoryInfo()
        actManager.getMemoryInfo(memInfo)
        if (cachedRamTotalMb == 0L) {
            cachedRamTotalMb = memInfo.totalMem / (1024 * 1024)
        }
        val ramTotalMb = cachedRamTotalMb
        val ramFreeMb = memInfo.availMem / (1024 * 1024)
        val ramUsedMb = (memInfo.totalMem - memInfo.availMem) / (1024 * 1024)

        // Storage Info
        val path = Environment.getDataDirectory()
        val stat = StatFs(path.path)
        val blockSize = stat.blockSizeLong
        val totalBlocks = stat.blockCountLong
        val availableBlocks = stat.availableBlocksLong
        if (cachedStorageTotalMb == 0L) {
            cachedStorageTotalMb = (totalBlocks * blockSize) / (1024 * 1024)
        }
        val storageTotalMb = cachedStorageTotalMb
        val storageFreeMb = (availableBlocks * blockSize) / (1024 * 1024)
        val storageUsedMb = storageTotalMb - storageFreeMb

        // Battery Info
        val batteryIntent = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val level = batteryIntent?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = batteryIntent?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        val batteryPct = if (level >= 0 && scale > 0) (level / scale.toFloat()) * 100f else 0f
        
        val status = batteryIntent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val isCharging = status == BatteryManager.BATTERY_STATUS_CHARGING || status == BatteryManager.BATTERY_STATUS_FULL
        
        val tempRaw = batteryIntent?.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, 0) ?: 0
        val batteryTempC = tempRaw / 10f

        val healthRaw = batteryIntent?.getIntExtra(BatteryManager.EXTRA_HEALTH, BatteryManager.BATTERY_HEALTH_UNKNOWN)
            ?: BatteryManager.BATTERY_HEALTH_UNKNOWN
        val batteryHealth = when (healthRaw) {
            BatteryManager.BATTERY_HEALTH_GOOD -> "Good"
            BatteryManager.BATTERY_HEALTH_OVERHEAT -> "Overheat"
            BatteryManager.BATTERY_HEALTH_DEAD -> "Dead"
            BatteryManager.BATTERY_HEALTH_OVER_VOLTAGE -> "Over Voltage"
            BatteryManager.BATTERY_HEALTH_COLD -> "Cold"
            else -> "Normal"
        }

        // Network Info
        val (ip, netType) = getNetworkInfo()

        return DeviceInfoState(
            rawModel = brandInfo.rawModel,
            brand = brandInfo.brand,
            product = product,
            marketingName = brandInfo.marketingName,
            displayName = brandInfo.bestDisplayName,
            androidVersion = androidVersion,
            androidSdk = androidSdk,
            kernelVersion = brandInfo.kernelVersion,
            socHardware = brandInfo.socHardware,
            ramTotalMb = ramTotalMb,
            ramUsedMb = ramUsedMb,
            ramFreeMb = ramFreeMb,
            storageTotalMb = storageTotalMb,
            storageUsedMb = storageUsedMb,
            storageFreeMb = storageFreeMb,
            batteryPercent = batteryPct,
            batteryCharging = isCharging,
            batteryTempC = batteryTempC,
            batteryHealth = batteryHealth,
            screenResolution = screenRes,
            screenRefreshRate = screenRefresh,
            ipAddress = ip,
            networkType = netType
        )
    }

    private fun getNetworkInfo(): Pair<String?, String> {
        return try {
            val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            val activeNetwork = cm?.activeNetwork
            val capabilities = cm?.getNetworkCapabilities(activeNetwork)

            val netType = when {
                capabilities == null -> "Offline"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> "WiFi"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> "Cellular"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> "Ethernet"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_BLUETOOTH) -> "Bluetooth"
                else -> "Connected"
            }

            // Find IPv4 address
            var ipAddress: String? = null
            val interfaces = NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val iface = interfaces.nextElement()
                if (iface.isUp && !iface.isLoopback) {
                    val addresses = iface.inetAddresses
                    while (addresses.hasMoreElements()) {
                        val addr = addresses.nextElement()
                        if (!addr.isLoopbackAddress && addr is Inet4Address) {
                            ipAddress = addr.hostAddress
                            break
                        }
                    }
                }
                if (ipAddress != null) break
            }

            Pair(ipAddress, netType)
        } catch (_: Exception) {
            Pair(null, "Unknown")
        }
    }

    fun toTelemetryRequest(): HardwareTelemetryRequest {
        val info = collectDeviceInfo()
        // Prioritize brand's marketing name if detected from file system / build.prop,
        // otherwise use raw model code.
        val modelForTelemetry = if (!info.marketingName.isNullOrBlank()) {
            info.displayName
        } else {
            info.rawModel
        }

        return HardwareTelemetryRequest(
            raw_model = modelForTelemetry,
            imei1 = null,
            imei2 = null,
            serial_number = null,
            ram_total_mb = info.ramTotalMb,
            ram_used_mb = info.ramUsedMb,
            ram_free_mb = info.ramFreeMb,
            storage_total_mb = info.storageTotalMb,
            storage_used_mb = info.storageUsedMb,
            storage_free_mb = info.storageFreeMb,
            battery_percent = info.batteryPercent,
            battery_charging = info.batteryCharging,
            battery_temp_c = info.batteryTempC,
            battery_health = info.batteryHealth,
            os_version = info.androidVersion,
            android_sdk = info.androidSdk,
            kernel_version = info.kernelVersion,
            screen_resolution = info.screenResolution,
            screen_refresh_rate = info.screenRefreshRate,
            ip_address = info.ipAddress,
            network_type = info.networkType
        )
    }
}
