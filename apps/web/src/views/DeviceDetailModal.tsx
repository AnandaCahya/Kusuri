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

  const specs = specsData?.specs;
  const telemetry = specsData?.telemetry;

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
            {specs?.picture_url ? (
              <img
                src={specs.picture_url}
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
                    <Cpu className="w-4 h-4 text-sky-400" /> Sistem Operasi
                  </div>
                  <div className="font-medium text-slate-200 text-sm">
                    {telemetry?.os_version || device.device_type}
                  </div>
                  {telemetry?.android_sdk && (
                    <span className="text-[11px] text-slate-500">
                      Android SDK: {telemetry.android_sdk}
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* GSM Arena Hardware Specs (Catalog Enricher) */}
            {specs ? (
              <div className="space-y-3">
                <h5 className="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                  <Cpu className="w-3.5 h-3.5 text-indigo-400" />
                  Spesifikasi Resmi (Katalog Pabrikan & GSM Arena)
                </h5>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  {specs.chipset && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Chipset</span>
                      <span className="text-slate-200 font-medium">{specs.chipset}</span>
                    </div>
                  )}

                  {specs.cpu && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">CPU & GPU</span>
                      <span className="text-slate-200 font-medium">{specs.cpu} {specs.gpu ? `(${specs.gpu})` : ""}</span>
                    </div>
                  )}

                  {specs.display_type && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Layar</span>
                      <span className="text-slate-200 font-medium">
                        {specs.display_type} • {specs.display_size}
                      </span>
                    </div>
                  )}

                  {specs.main_camera && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Kamera Utama</span>
                      <span className="text-slate-200 font-medium">{specs.main_camera}</span>
                    </div>
                  )}

                  {specs.battery_type && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Kapasitas Baterai Pabrik</span>
                      <span className="text-slate-200 font-medium">
                        {specs.battery_type} {specs.battery_charging ? `(${specs.battery_charging})` : ""}
                      </span>
                    </div>
                  )}

                  {specs.dimensions && (
                    <div className="p-3 rounded-xl bg-slate-950/40 border border-slate-800">
                      <span className="text-slate-500 block">Dimensi & Berat</span>
                      <span className="text-slate-200 font-medium">
                        {specs.dimensions} {specs.weight ? `• ${specs.weight}` : ""}
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
