package com.anandacahya.kusuri.data.telemetry

import android.annotation.SuppressLint
import android.content.Context
import android.os.Build
import android.provider.Settings
import android.util.Log
import java.io.BufferedReader
import java.io.File
import java.io.FileReader
import java.util.Locale

data class BrandDeviceInfo(
    val brand: String,
    val manufacturer: String,
    val rawModel: String,
    val marketingName: String?,
    val bestDisplayName: String,
    val codename: String,
    val kernelVersion: String?,
    val socHardware: String?
)

object BrandDeviceResolver {
    private const val TAG = "BrandDeviceResolver"

    @Volatile
    private var cachedInfo: BrandDeviceInfo? = null

    // Known property keys across Android manufacturers
    private val MARKET_NAME_PROP_KEYS = listOf(
        // Transsion (Infinix, Tecno, Itel)
        "ro.transsion.market.name",
        "ro.transsion.product.name",
        "ro.product.brand.market",
        // Xiaomi / Redmi / Poco
        "ro.product.marketname",
        "ro.product.odm.marketname",
        "ro.product.vendor.marketname",
        "ro.product.system.marketname",
        // Oppo / Realme / OnePlus
        "ro.oppo.market.name",
        "ro.realme.market.name",
        "ro.oplus.market.name",
        "ro.vendor.oplus.marketname",
        // Vivo / iQOO
        "ro.vivo.market.name",
        // Huawei / Honor
        "ro.config.marketing_name",
        "ro.product.hw_model",
        // Samsung & General Android
        "ro.product.vendor.marketname",
        "ro.vendor.marketname",
        "ro.product.model.display",
        "ro.product.display",
        "ro.vendor.display.model",
        "ro.vendor.product.display"
    )

    private val BUILD_PROP_PATHS = listOf(
        "/system/build.prop",
        "/vendor/build.prop",
        "/product/build.prop",
        "/system_ext/build.prop",
        "/odm/etc/build.prop",
        "/vendor/etc/build.prop",
        "/system/etc/prop.default",
        "/default.prop"
    )

    fun resolve(context: Context): BrandDeviceInfo {
        cachedInfo?.let { return it }
        synchronized(this) {
            cachedInfo?.let { return it }

            val rawModel = Build.MODEL ?: "Unknown"
            val rawBrand = Build.BRAND ?: "Unknown"
            val manufacturer = Build.MANUFACTURER ?: "Unknown"
            val codename = Build.DEVICE ?: "Unknown"

            // 1. Try reading marketing name from system properties (reflection)
            var marketingName: String? = null
            for (key in MARKET_NAME_PROP_KEYS) {
                val propVal = getSystemProperty(key)?.trim()
                if (!propVal.isNullOrBlank() && !propVal.equals("unknown", ignoreCase = true)) {
                    marketingName = propVal
                    Log.d(TAG, "Found marketing name via SystemProperty '$key': $marketingName")
                    break
                }
            }

            // 2. If not found, inspect readable build.prop files on the filesystem
            if (marketingName.isNullOrBlank()) {
                val fileProps = readBuildPropFiles()
                for (key in MARKET_NAME_PROP_KEYS) {
                    val propVal = fileProps[key]?.trim()
                    if (!propVal.isNullOrBlank() && !propVal.equals("unknown", ignoreCase = true)) {
                        marketingName = propVal
                        Log.d(TAG, "Found marketing name from filesystem build.prop '$key': $marketingName")
                        break
                    }
                }

                // If still not found, check any key containing 'marketname' or 'market.name'
                if (marketingName.isNullOrBlank()) {
                    for ((k, v) in fileProps) {
                        val lower = k.lowercase(Locale.ROOT)
                        if ((lower.contains("marketname") || lower.contains("market.name") || lower.contains("marketing_name")) &&
                            v.isNotBlank() && !v.equals("unknown", ignoreCase = true)
                        ) {
                            marketingName = v.trim()
                            Log.d(TAG, "Found marketing name via fuzzy filesystem prop '$k': $marketingName")
                            break
                        }
                    }
                }
            }

            // 3. Check Settings.Global / Settings.System device_name
            val settingsDeviceName = getSettingsDeviceName(context)

            // Normalize Brand representation
            val brand = formatBrand(rawBrand, manufacturer)

            // 4. Construct best retail display name
            val bestDisplayName = when {
                !marketingName.isNullOrBlank() -> {
                    if (marketingName.lowercase(Locale.ROOT).contains(brand.lowercase(Locale.ROOT))) {
                        capitalizeWords(marketingName)
                    } else {
                        "$brand ${capitalizeWords(marketingName)}"
                    }
                }
                !settingsDeviceName.isNullOrBlank() && !isGenericDeviceName(settingsDeviceName) -> {
                    settingsDeviceName
                }
                rawModel.lowercase(Locale.ROOT).startsWith(brand.lowercase(Locale.ROOT)) -> {
                    capitalizeWords(rawModel)
                }
                else -> {
                    "$brand $rawModel"
                }
            }

            val kernel = readKernelVersion()
            val soc = readSocHardware()

            val info = BrandDeviceInfo(
                brand = brand,
                manufacturer = manufacturer,
                rawModel = rawModel,
                marketingName = marketingName,
                bestDisplayName = bestDisplayName,
                codename = codename,
                kernelVersion = kernel,
                socHardware = soc
            )

            cachedInfo = info
            return info
        }
    }

    private fun getSystemProperty(key: String): String? {
        return try {
            val clazz = Class.forName("android.os.SystemProperties")
            val method = clazz.getMethod("get", String::class.java)
            val result = method.invoke(null, key) as? String
            if (result.isNullOrBlank()) null else result
        } catch (_: Throwable) {
            null
        }
    }

    private fun readBuildPropFiles(): Map<String, String> {
        val map = mutableMapOf<String, String>()
        for (path in BUILD_PROP_PATHS) {
            val file = File(path)
            if (file.exists() && file.canRead()) {
                try {
                    BufferedReader(FileReader(file)).use { reader ->
                        var line = reader.readLine()
                        while (line != null) {
                            val trimmed = line.trim()
                            if (!trimmed.startsWith("#") && trimmed.contains("=")) {
                                val split = trimmed.split("=", limit = 2)
                                if (split.size == 2) {
                                    val key = split[0].trim()
                                    val value = split[1].trim()
                                    if (key.isNotEmpty() && value.isNotEmpty()) {
                                        map[key] = value
                                    }
                                }
                            }
                            line = reader.readLine()
                        }
                    }
                } catch (e: Exception) {
                    Log.v(TAG, "Cannot read $path: ${e.message}")
                }
            }
        }
        return map
    }

    @SuppressLint("HardwareIds")
    private fun getSettingsDeviceName(context: Context): String? {
        return try {
            val name = Settings.Global.getString(context.contentResolver, "device_name")
                ?: Settings.System.getString(context.contentResolver, "device_name")
                ?: Settings.Secure.getString(context.contentResolver, "bluetooth_name")
            name?.trim()?.takeIf { it.isNotBlank() }
        } catch (_: Throwable) {
            null
        }
    }

    private fun isGenericDeviceName(name: String): Boolean {
        val lower = name.lowercase(Locale.ROOT)
        return lower == "android" || lower == "localhost" || lower == "phone" || lower == "tablet" || lower.startsWith("android_")
    }

    private fun readKernelVersion(): String? {
        return try {
            val file = File("/proc/version")
            if (file.exists() && file.canRead()) {
                val line = file.readText().trim()
                // Example: "Linux version 5.10.168-android12-9-..."
                // Extract version up to space before parenthesis or hash
                val regex = Regex("""Linux version (\S+)""")
                val match = regex.find(line)
                match?.groupValues?.get(1) ?: line.take(40)
            } else {
                System.getProperty("os.version")
            }
        } catch (_: Throwable) {
            System.getProperty("os.version")
        }
    }

    private fun readSocHardware(): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val soc = Build.SOC_MODEL
                if (!soc.isNullOrBlank() && !soc.equals("unknown", ignoreCase = true)) {
                    return soc
                }
            }
            val file = File("/proc/cpuinfo")
            if (file.exists() && file.canRead()) {
                var foundHw: String? = null
                file.useLines { lines ->
                    for (line in lines) {
                        if (line.startsWith("Hardware", ignoreCase = true) && line.contains(":")) {
                            val hw = line.substringAfter(":").trim()
                            if (hw.isNotBlank()) {
                                foundHw = hw
                                break
                            }
                        }
                    }
                }
                if (!foundHw.isNullOrBlank()) {
                    return foundHw
                }
            }
            Build.HARDWARE?.takeIf { it.isNotBlank() && !it.equals("unknown", ignoreCase = true) }
        } catch (_: Throwable) {
            Build.HARDWARE
        }
    }

    private fun formatBrand(brand: String, manufacturer: String): String {
        val candidate = if (brand.isNotBlank() && !brand.equals("unknown", ignoreCase = true)) {
            brand
        } else {
            manufacturer
        }
        return capitalizeWords(candidate)
    }

    private fun capitalizeWords(str: String): String {
        return str.split(" ").joinToString(" ") { word ->
            word.replaceFirstChar { if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString() }
        }
    }
}
