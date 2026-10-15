import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useToast } from "../context/ToastContext";
import { CardSkeleton } from "../components/common/Skeleton";
import type { SystemHealth } from "../types";
import {
  Server,
  Activity,
  Database,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Cpu,
  Layers,
  ShieldAlert,
  Loader2,
  HardDrive,
} from "lucide-react";

export const AdminView: React.FC = () => {
  const { addToast } = useToast();

  const [health, setHealth] = useState<SystemHealth | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSyncingCatalog, setIsSyncingCatalog] = useState(false);

  const fetchHealth = async () => {
    setIsLoading(true);
    try {
      const data = await api.system.health();
      setHealth(data);
    } catch (err: any) {
      addToast(err.message || "Gagal memuat status kesehatan server", "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchHealth();
  }, []);

  const handleSyncCatalog = async () => {
    setIsSyncingCatalog(true);
    try {
      const res = await api.devices.syncCatalog();
      addToast(
        res.message || "Katalog Google Play Supported Devices berhasil disinkronkan!",
        "success"
      );
    } catch (err: any) {
      addToast(err.message || "Gagal menyinkronkan katalog perangkat", "error");
    } finally {
      setIsSyncingCatalog(false);
    }
  };

  return (
    <div className="max-w-5xl mx-auto space-y-8 animate-in fade-in duration-200">
      {/* Admin Title */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2.5">
            <Server className="w-6 h-6 text-indigo-400" />
            Instance Administration
          </h2>
          <p className="text-sm text-slate-400 mt-1">
            Pemantauan kesehatan sistem dan operasional instance Kusuri self-hosted
          </p>
        </div>

        <button
          onClick={fetchHealth}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 bg-slate-900 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-white rounded-xl text-xs font-medium transition cursor-pointer"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? "animate-spin" : ""}`} />
          <span>Periksa Ulang</span>
        </button>
      </div>

      {/* System Health Card */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Status */}
        <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>Status Backend</span>
            <Activity className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="flex items-center gap-2 text-xl font-bold text-white capitalize">
            {health?.status === "healthy" ? (
              <>
                <CheckCircle2 className="w-5 h-5 text-emerald-400" />
                <span className="text-emerald-400">Sehat (Healthy)</span>
              </>
            ) : (
              <>
                <AlertCircle className="w-5 h-5 text-rose-400" />
                <span className="text-rose-400">Perlu Pemeriksaan</span>
              </>
            )}
          </div>
          <p className="text-xs text-slate-500">Service: {health?.service || "kusuri-backend"}</p>
        </div>

        {/* Version */}
        <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>Versi Rust Engine</span>
            <Cpu className="w-4 h-4 text-indigo-400" />
          </div>
          <div className="text-xl font-bold text-white font-mono">
            v{health?.version || "0.1.0"}
          </div>
          <p className="text-xs text-slate-500">Axum + Tokio + SQLx</p>
        </div>

        {/* Database */}
        <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>Konektivitas Database</span>
            <Database className="w-4 h-4 text-sky-400" />
          </div>
          <div className="text-xl font-bold text-white">PostgreSQL</div>
          <p className="text-xs text-slate-500">Migrasi skema otomatis aktif</p>
        </div>
      </div>

      {/* Device Catalog Sync Action */}
      <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-4">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div className="space-y-1">
            <h3 className="text-base font-semibold text-slate-100 flex items-center gap-2">
              <HardDrive className="w-4 h-4 text-indigo-400" />
              Katalog Google Play Supported Devices
            </h3>
            <p className="text-xs text-slate-400 max-w-xl leading-relaxed">
              Kusuri mengunduh dan memetakan model hardware mentah (contoh: <code className="text-indigo-300">Infinix-X6881</code>) ke nama komersial resmi (<code className="text-indigo-300">Infinix HOT 50 Pro</code>) serta cache GSM Arena.
            </p>
          </div>

          <button
            onClick={handleSyncCatalog}
            disabled={isSyncingCatalog}
            className="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium text-xs rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer shrink-0"
          >
            {isSyncingCatalog ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <RefreshCw className="w-4 h-4" />
            )}
            <span>{isSyncingCatalog ? "Menyinkronkan..." : "Sinkronisasi Katalog Sekarang"}</span>
          </button>
        </div>
      </div>

      {/* REQ.md Section 17 & Section 7 Compliance Note */}
      <div className="p-6 rounded-3xl bg-slate-950/80 border border-slate-800/80 space-y-3 text-xs">
        <div className="flex items-center gap-2 text-amber-400 font-semibold text-sm">
          <ShieldAlert className="w-4 h-4" />
          <span>Kepatuhan Spesifikasi REQ.md (Prinsip Desain & Batasan)</span>
        </div>
        <p className="text-slate-400 leading-relaxed">
          Sesuai ketentuan dalam <strong>REQ.md Seksi 17</strong>, antarmuka web tidak mengasumsikan endpoint, permission, atau metrik yang belum diimplementasikan di server backend.
        </p>
        <ul className="list-disc list-inside space-y-1 text-slate-400">
          <li>Pengelolaan pengguna lintas instance dan audit log terminal arbitrer sengaja tidak diaktifkan karena backend saat ini difokuskan sebagai personal self-hosted control plane.</li>
          <li>Koneksi real-time dilindungi secara ketat melalui tiket JWT otentikasi tanpa mengekspos token rahasia ke URL log browser.</li>
        </ul>
      </div>
    </div>
  );
};
