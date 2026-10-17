package com.anandacahya.kusuri.data.telemetry

import android.app.ActivityManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import android.os.Build
import android.os.Environment
import android.os.StatFs
import com.anandacahya.kusuri.data.model.HardwareTelemetryRequest

data class DeviceInfoState(
    val rawModel: String,
    val brand: String,
    val product: String,
    val androidVersion: String,
    val androidSdk: Int,
    val ramTotalMb: Long,
    val ramUsedMb: Long,
    val storageTotalMb: Long,
    val storageUsedMb: Long,
    val batteryPercent: Float,
    val batteryCharging: Boolean,
    val batteryTempC: Float,
    val screenResolution: String = "",
    val screenRefreshRate: Float = 60f
) {
    val sdkInt: Int get() = androidSdk
}

class DeviceTelemetryCollector(private val context: Context) {

    fun collectDeviceInfo(): DeviceInfoState {
        val rawModel = Build.MODEL ?: "Unknown"
        val brand = Build.BRAND ?: "Unknown"
        val product = Build.PRODUCT ?: "Unknown"
        val androidVersion = "Android ${Build.VERSION.RELEASE}"
        val sdk = Build.VERSION.SDK_INT

        // RAM Info
        val actManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val memInfo = ActivityManager.MemoryInfo()
        actManager.getMemoryInfo(memInfo)
        val ramTotalMb = memInfo.totalMem / (1024 * 1024)
        val ramUsedMb = (memInfo.totalMem - memInfo.availMem) / (1024 * 1024)

        // Storage Info
        val path = Environment.getDataDirectory()
        val stat = StatFs(path.path)
        val blockSize = stat.blockSizeLong
        val totalBlocks = stat.blockCountLong
        val availableBlocks = stat.availableBlocksLong
        val storageTotalMb = (totalBlocks * blockSize) / (1024 * 1024)
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

        val dm = context.resources.displayMetrics
        val screenRes = "${dm.widthPixels} x ${dm.heightPixels}"
        val refreshRate = try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                context.display?.refreshRate ?: 60.0f
            } else {
                60.0f
            }
        } catch (_: Exception) {
            60.0f
        }

        return DeviceInfoState(
            rawModel = rawModel,
            brand = brand,
            product = product,
            androidVersion = androidVersion,
            androidSdk = sdk,
            ramTotalMb = ramTotalMb,
            ramUsedMb = ramUsedMb,
            storageTotalMb = storageTotalMb,
            storageUsedMb = storageUsedMb,
            batteryPercent = batteryPct,
            batteryCharging = isCharging,
            batteryTempC = batteryTempC,
            screenResolution = screenRes,
            screenRefreshRate = refreshRate
        )
    }

    fun toTelemetryRequest(): HardwareTelemetryRequest {
        val info = collectDeviceInfo()
        return HardwareTelemetryRequest(
            raw_model = info.rawModel,
            imei1 = null,
            imei2 = null,
            ram_total_mb = info.ramTotalMb,
            ram_used_mb = info.ramUsedMb,
            storage_total_mb = info.storageTotalMb,
            storage_used_mb = info.storageUsedMb,
            battery_percent = info.batteryPercent,
            battery_charging = info.batteryCharging,
            battery_temp_c = info.batteryTempC,
            os_version = info.androidVersion,
            android_sdk = info.androidSdk,
            screen_resolution = info.screenResolution,
            screen_refresh_rate = info.screenRefreshRate
        )
    }
}
