import React, { useEffect, useState } from "react";
import { Modal } from "../components/common/Modal";
import { api } from "../services/api";
import { StatusBadge, DeviceTypeBadge } from "../components/common/Badge";
import type { Device, EnrichedDeviceSpecsResponse } from "../types";
import {
  Cpu,
  Monitor,
  Camera,
  Battery,
  BatteryCharging,
  HardDrive,
  Layers,
  Thermometer,
  Shield,
  Eye,
  EyeOff,
  Clock,
  Loader2,
  RefreshCw,
  Sparkles,
  ExternalLink,
  CheckCircle2,
} from "lucide-react";

interface DeviceDetailModalProps {
  device: Device | null;
  isOpen: boolean;
  onClose: () => void;
}

export const DeviceDetailModal: React.FC<DeviceDetailModalProps> = ({
  device,
  isOpen,
  onClose,
}) => {
  const [specsData, setSpecsData] = useState<EnrichedDeviceSpecsResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [showImei, setShowImei] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!device || !isOpen) {
      setSpecsData(null);
      setShowImei(false);
      setError(null);
      return;
    }

    const fetchSpecs = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const data = await api.devices.getSpecs(device.id);
        setSpecsData(data);
      } catch (err: any) {
        setError(err.message || "Gagal memuat detail spesifikasi perangkat");
      } finally {
        setIsLoading(false);
      }
    };

    fetchSpecs();
  }, [device, isOpen]);

  if (!device) return null;

  const specs = specsData?.specs || specsData?.catalog_specs;
  const telemetry = specsData?.telemetry || specsData?.live_hardware;
  const imageUrl = specs?.picture_url || specs?.image_url;
  const verifiedSources = specs?.raw_specs?.sources || (specs?.gsmarena_url ? ["GSM Arena"] : []);
  const sourceUrls = specs?.raw_specs?.source_urls || (specs?.gsmarena_url ? { "GSM Arena": specs.gsmarena_url } : {});

  const ramTotalMb = telemetry?.ram_total_mb || 0;
  const ramUsedMb = telemetry?.ram_used_mb || 0;
  const ramPercent = ramTotalMb > 0 ? Math.round((ramUsedMb / ramTotalMb) * 100) : null;

  const storageTotalMb = telemetry?.storage_total_mb || 0;
  const storageUsedMb = telemetry?.storage_used_mb || 0;
  const storagePercent =
    storageTotalMb > 0 ? Math.round((storageUsedMb / storageTotalMb) * 100) : null;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={device.marketing_name || device.name}
      maxWidth="2xl"
    >
      <div className="space-y-6">
        {/* Header Summary */}
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 p-4 rounded-2xl bg-slate-950/70 border border-slate-800">
          <div className="flex items-center gap-3">
            {imageUrl ? (
              <img
                src={imageUrl}
                alt={device.name}
                className="w-16 h-16 object-contain bg-slate-900 rounded-xl p-1 border border-slate-800"
              />
            ) : (
              <div className="w-14 h-14 rounded-xl bg-indigo-600/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                <Cpu className="w-7 h-7" />
              </div>
            )}
            <div>
              <div className="flex items-center gap-2">
                <h4 className="font-semibold text-white text-base">
                  {device.marketing_name || device.name}
                </h4>
                <StatusBadge isOnline={device.is_online} />
              </div>
              <p className="text-xs text-slate-400 mt-0.5">
                {device.retail_branding ? `${device.retail_branding} • ` : ""}
                Codename: <span className="font-mono text-slate-300">{device.raw_model || device.name}</span>
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <DeviceTypeBadge type={device.device_type} />
          </div>
        </div>

        {isLoading && (
          <div className="flex flex-col items-center justify-center py-12 gap-3 text-slate-400">
            <Loader2 className="w-6 h-6 animate-spin text-indigo-400" />
            <span className="text-xs">Memuat katalog & telemetri hardware...</span>
          </div>
        )}

        {error && (
          <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-xs">
            {error}
          </div>
        )}

        {!isLoading && (
          <>
            {/* Live Hardware Telemetry Section */}
            <div className="space-y-3">
              <h5 className="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
                Live Telemetri Perangkat
              </h5>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {/* RAM Usage */}
                <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
                  <div className="flex items-center justify-between text-xs mb-2">
                    <span className="text-slate-400 flex items-center gap-1.5">
                      <Layers className="w-4 h-4 text-indigo-400" /> Penggunaan RAM
                    </span>
                    <span className="font-semibold text-slate-200">
                      {ramPercent !== null ? `${ramPercent}%` : "Unavailable"}
                    </span>
                  </div>
                  {ramTotalMb > 0 ? (
                    <>
                      <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden mb-1.5">
                        <div
                          className="bg-indigo-500 h-full rounded-full transition-all duration-500"
                          style={{ width: `${Math.min(ramPercent || 0, 100)}%` }}
                        />
                      </div>
                      <span className="text-[11px] text-slate-500">
                        {(ramUsedMb / 1024).toFixed(1)} GB / {(ramTotalMb / 1024).toFixed(1)} GB
                      </span>
                    </>
                  ) : (
                    <span className="text-xs text-slate-500 italic">Data belum dilaporkan</span>
                  )}
                </div>

                {/* Storage Usage */}
                <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
                  <div className="flex items-center justify-between text-xs mb-2">
                    <span className="text-slate-400 flex items-center gap-1.5">
                      <HardDrive className="w-4 h-4 text-violet-400" /> Penggunaan Storage
                    </span>
                    <span className="font-semibold text-slate-200">
                      {storagePercent !== null ? `${storagePercent}%` : "Unavailable"}
                    </span>
                  </div>
                  {storageTotalMb > 0 ? (
                    <>
                      <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden mb-1.5">
                        <div
                          className="bg-violet-500 h-full rounded-full transition-all duration-500"
                          style={{ width: `${Math.min(storagePercent || 0, 100)}%` }}
                        />
                      </div>
                      <span className="text-[11px] text-slate-500">
                        {(storageUsedMb / 1024).toFixed(1)} GB / {(storageTotalMb / 1024).toFixed(1)} GB
                      </span>
                    </>
                  ) : (
                    <span className="text-xs text-slate-500 italic">Data belum dilaporkan</span>
                  )}
                </div>

                {/* Battery Status */}
                <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
                  <div className="flex items-center justify-between text-xs mb-2">
                    <span className="text-slate-400 flex items-center gap-1.5">
                      {telemetry?.battery_charging ? (
                        <BatteryCharging className="w-4 h-4 text-emerald-400" />
                      ) : (
                        <Battery className="w-4 h-4 text-emerald-400" />
                      )}
                      Baterai
                    </span>
                    <span className="font-semibold text-slate-200">
                      {telemetry?.battery_percent !== undefined && telemetry?.battery_percent !== null
                        ? `${telemetry.battery_percent}%`
                        : "Unavailable"}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 text-[11px] text-slate-400">
                    <span>{telemetry?.battery_charging ? "Sedang Mengisi Daya (Charging)" : "Baterai"}</span>
                    {telemetry?.battery_temp_c && (
                      <>
                        <span>•</span>
                        <span className="flex items-center gap-1">
                          <Thermometer className="w-3 h-3 text-amber-400" />
                          {telemetry.battery_temp_c}°C
                        </span>
                      </>
                    )}
                  </div>
                </div>

                {/* OS Details */}
                <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
                  <div className="text-xs text-slate-400 mb-1 flex items-center gap-1.5">
                    <Cpu className="w-4 h-4 text-sky-400" /> Sistem Operasi & Host
                  </div>
                  <div className="font-medium text-slate-200 text-sm">
                    {telemetry?.os_version || device.device_type}
                  </div>
                  <div className="flex flex-wrap items-center gap-2 mt-1 text-[11px] text-slate-400">
                    {telemetry?.architecture && (
                      <span className="px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 font-mono">
                        {telemetry.architecture}
                      </span>
                    )}
                    {telemetry?.hostname && (
                      <span>Host: <span className="font-mono text-slate-300">{telemetry.hostname}</span></span>
                    )}
                    {telemetry?.uptime_formatted && (
                      <span>• Uptime: {telemetry.uptime_formatted}</span>
                    )}
                    {telemetry?.android_sdk && (
                      <span>Android SDK: {telemetry.android_sdk}</span>
                    )}
                  </div>
                </div>

                {/* Desktop Hardware specifics: CPU & GPU */}
                {(telemetry?.cpu_model || telemetry?.gpu_model) && (
                  <div className="sm:col-span-2 p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 space-y-2">
                    <div className="text-xs text-slate-400 flex items-center gap-1.5">
                      <Cpu className="w-4 h-4 text-emerald-400" /> Prosesor & Grafis
                    </div>
                    {telemetry.cpu_model && (
                      <div className="text-xs">
                        <span className="text-slate-400">CPU: </span>
                        <span className="font-medium text-slate-200">{telemetry.cpu_model}</span>
                        {telemetry.cpu_cores && (
                          <span className="text-slate-400 ml-1.5">
                            ({telemetry.cpu_cores} Cores / {telemetry.cpu_logical_cores || telemetry.cpu_cores} Threads
                            {telemetry.cpu_frequency_mhz ? ` @ ${telemetry.cpu_frequency_mhz} MHz` : ""})
                          </span>
                        )}
                      </div>
                    )}
                    {telemetry.gpu_model && (
                      <div className="text-xs">
                        <span className="text-slate-400">GPU / Display Adapter: </span>
                        <span className="font-medium text-slate-200">{telemetry.gpu_model}</span>
                      </div>
                    )}
                  </div>
                )}

                {/* Desktop Disks / Partition Breakdown */}
                {telemetry?.disks_info && (
                  <div className="sm:col-span-2 p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 space-y-2">
                    <div className="text-xs text-slate-400 flex items-center gap-1.5">
                      <HardDrive className="w-4 h-4 text-violet-400" /> Partisi Penyimpanan (Disks)
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                      {Array.isArray(telemetry.disks_info) ? (
                        telemetry.disks_info.map((d: any, idx: number) => {
                          const diskTotal = d.total_mb || 0;
                          const diskFree = d.free_mb || 0;
                          const diskUsed = diskTotal - diskFree;
                          const diskPct = diskTotal > 0 ? Math.round((diskUsed / diskTotal) * 100) : 0;
                          return (
                            <div key={idx} className="p-2.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs">
                              <div className="flex justify-between items-center font-medium text-slate-200">
                                <span>{d.name || d.mount_point || `Drive ${idx + 1}`}</span>
                                <span className="text-[10px] text-slate-400">{d.file_system || ""}</span>
                              </div>
                              <div className="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden my-1.5">
                                <div
                                  className="bg-violet-500 h-full rounded-full"
                                  style={{ width: `${Math.min(diskPct, 100)}%` }}
                                />
                              </div>
                              <div className="text-[10px] text-slate-400 flex justify-between">
                                <span>{(diskUsed / 1024).toFixed(1)} GB used</span>
                                <span>{(diskTotal / 1024).toFixed(1)} GB total</span>
                              </div>
                            </div>
                          );
                        })
                      ) : (
                        <div className="text-xs text-slate-300 font-mono">
                          {typeof telemetry.disks_info === "string" ? telemetry.disks_info : JSON.stringify(telemetry.disks_info)}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Collective Multi-Source Hardware Specs */}
            {specs ? (
              <div className="space-y-3">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                  <h5 className="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                    <Cpu className="w-3.5 h-3.5 text-indigo-400" />
                    Spesifikasi Kolektif Terverifikasi
                  </h5>
                  {verifiedSources.length > 0 && (
                    <div className="flex flex-wrap items-center gap-1.5">
                      <span className="text-[11px] text-slate-500 flex items-center gap-1">
                        <CheckCircle2 className="w-3 h-3 text-emerald-400" /> Sumber:
                      </span>
                      {verifiedSources.map((src) => {
                        const url = sourceUrls[src];
                        return url ? (
                          <a
                            key={src}
                            href={url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 hover:bg-emerald-500/20 transition-colors"
                          >
                            <span>{src}</span>
                            <ExternalLink className="w-2.5 h-2.5 opacity-70" />
                          </a>
                        ) : (
                          <span
                            key={src}
                            className="px-2 py-0.5 rounded-full text-[10px] font-medium bg-indigo-500/10 text-indigo-300 border border-indigo-500/20"
                          >
                            {src}
                          </span>
                        );
                      })}
                    </div>
                  )}
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  {specs.chipset && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Chipset</span>
                      <span className="text-slate-200 font-medium">{specs.chipset}</span>
                    </div>
                  )}

                  {(specs.cpu || specs.gpu) && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">CPU & GPU</span>
                      <span className="text-slate-200 font-medium">
                        {specs.cpu || "-"} {specs.gpu ? `(${specs.gpu})` : ""}
                      </span>
                    </div>
                  )}

                  {(specs.display_type || specs.display_size || specs.display_resolution) && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Layar</span>
                      <span className="text-slate-200 font-medium">
                        {[specs.display_type, specs.display_size, specs.display_resolution]
                          .filter(Boolean)
                          .join(" • ")}
                      </span>
                    </div>
                  )}

                  {(specs.main_camera || specs.camera_spec) && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Kamera Utama</span>
                      <span className="text-slate-200 font-medium">{specs.main_camera || specs.camera_spec}</span>
                    </div>
                  )}

                  {(specs.battery_type || specs.battery_spec) && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Kapasitas Baterai Pabrik</span>
                      <span className="text-slate-200 font-medium">
                        {specs.battery_type || specs.battery_spec}{" "}
                        {specs.battery_charging || specs.charging_spec ? `(${specs.battery_charging || specs.charging_spec})` : ""}
                      </span>
                    </div>
                  )}

                  {specs.memory_spec && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Pilihan RAM & Storage Pabrik</span>
                      <span className="text-slate-200 font-medium">{specs.memory_spec}</span>
                    </div>
                  )}

                  {specs.os_original && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">OS Asli Bawaan</span>
                      <span className="text-slate-200 font-medium">{specs.os_original}</span>
                    </div>
                  )}

                  {(specs.dimensions || specs.weight) && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Dimensi & Berat</span>
                      <span className="text-slate-200 font-medium">
                        {specs.dimensions || "-"} {specs.weight ? `• ${specs.weight}` : ""}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            ) : null}

            {/* Privacy Section: Protected IMEI (Per REQ.md Sec. 6.4: hidden by default) */}
            {(telemetry?.imei1 || telemetry?.imei2) && (
              <div className="p-4 rounded-2xl bg-slate-950/80 border border-slate-800/90 space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-semibold text-slate-400 flex items-center gap-1.5">
                    <Shield className="w-3.5 h-3.5 text-amber-400" />
                    Identitas Perangkat Sensitif (IMEI)
                  </span>
                  <button
                    type="button"
                    onClick={() => setShowImei(!showImei)}
                    className="flex items-center gap-1.5 text-xs text-indigo-400 hover:text-indigo-300 cursor-pointer"
                  >
                    {showImei ? (
                      <>
                        <EyeOff className="w-3.5 h-3.5" />
                        <span>Sembunyikan</span>
                      </>
                    ) : (
                      <>
                        <Eye className="w-3.5 h-3.5" />
                        <span>Tampilkan</span>
                      </>
                    )}
                  </button>
                </div>
                <div className="text-xs font-mono text-slate-300">
                  {showImei ? (
                    <div>
                      {telemetry.imei1 && <p>IMEI 1: {telemetry.imei1}</p>}
                      {telemetry.imei2 && <p>IMEI 2: {telemetry.imei2}</p>}
                    </div>
                  ) : (
                    <p className="text-slate-500 italic">Disembunyikan demi privasi</p>
                  )}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </Modal>
  );
};
