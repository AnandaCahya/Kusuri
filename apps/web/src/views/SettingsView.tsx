import React, { useState } from "react";
import { useAuth } from "../context/AuthContext";
import { useToast } from "../context/ToastContext";
import { getApiBaseUrl, setApiBaseUrl, getWsBaseUrl } from "../services/config";
import {
  User,
  Shield,
  Server,
  Lock,
  LogOut,
  Save,
  Info,
  CheckCircle2,
  Key,
} from "lucide-react";

export const SettingsView: React.FC = () => {
  const { user, token, logout } = useAuth();
  const { addToast } = useToast();

  const [apiUrl, setApiUrlState] = useState(getApiBaseUrl());
  const [isSaved, setIsSaved] = useState(false);

  const handleSaveApiUrl = (e: React.FormEvent) => {
    e.preventDefault();
    setApiBaseUrl(apiUrl.trim());
    setIsSaved(true);
    addToast("Konfigurasi URL backend berhasil disimpan! Memuat ulang...", "success");
    setTimeout(() => {
      window.location.reload();
    }, 1000);
  };

  const maskedToken = token
    ? `${token.substring(0, 10)}...${token.substring(token.length - 8)}`
    : "Tidak ada token";

  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-in fade-in duration-200">
      {/* Account Info Section */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 sm:p-8 space-y-6">
        <div className="flex items-center gap-3 pb-4 border-b border-slate-800">
          <div className="p-3 rounded-2xl bg-indigo-600/10 text-indigo-400 border border-indigo-500/20">
            <User className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-semibold text-slate-100 text-base">Profil Akun</h3>
            <p className="text-xs text-slate-400">Informasi identitas akun pengguna Kusuri</p>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
          <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
            <span className="text-slate-500 block mb-1">Username</span>
            <span className="text-slate-200 font-medium text-sm">{user?.username}</span>
          </div>

          <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
            <span className="text-slate-500 block mb-1">User ID (UUID)</span>
            <span className="text-slate-300 font-mono text-xs break-all">{user?.id}</span>
          </div>

          <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
            <span className="text-slate-500 block mb-1">Waktu Pendaftaran Akun</span>
            <span className="text-slate-200">
              {user?.created_at ? new Date(user.created_at).toLocaleString() : "-"}
            </span>
          </div>

          <div className="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80">
            <span className="text-slate-500 block mb-1">Status Sesi JWT</span>
            <span className="text-emerald-400 font-medium flex items-center gap-1.5">
              <CheckCircle2 className="w-3.5 h-3.5" />
              Aktif & Terotentikasi
            </span>
          </div>
        </div>
      </div>

      {/* Backend & Deployment Configuration Section */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 sm:p-8 space-y-6">
        <div className="flex items-center gap-3 pb-4 border-b border-slate-800">
          <div className="p-3 rounded-2xl bg-sky-600/10 text-sky-400 border border-sky-500/20">
            <Server className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-semibold text-slate-100 text-base">
              Konfigurasi Server Self-Hosted
            </h3>
            <p className="text-xs text-slate-400">
              Sesuaikan endpoint REST API & WebSocket bila Kusuri Backend di-host pada server lain
            </p>
          </div>
        </div>

        <form onSubmit={handleSaveApiUrl} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Backend API Base URL
            </label>
            <input
              type="text"
              value={apiUrl}
              onChange={(e) => setApiUrlState(e.target.value)}
              placeholder="http://localhost:8080 atau https://api.kusuri.domain"
              required
              className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-4 py-2.5 text-sm text-slate-100 font-mono placeholder:text-slate-600 focus:outline-none transition"
            />
            <p className="text-[11px] text-slate-500 mt-1">
              WebSocket target otomatis: <span className="font-mono text-slate-400">{getWsBaseUrl()}</span>
            </p>
          </div>

          <div className="flex justify-end">
            <button
              type="submit"
              className="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer"
            >
              <Save className="w-4 h-4" />
              <span>Simpan & Terapkan Endpoint</span>
            </button>
          </div>
        </form>
      </div>

      {/* Security & Token Policy Section */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 sm:p-8 space-y-6">
        <div className="flex items-center gap-3 pb-4 border-b border-slate-800">
          <div className="p-3 rounded-2xl bg-emerald-600/10 text-emerald-400 border border-emerald-500/20">
            <Shield className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-semibold text-slate-100 text-base">
              Keamanan & Perlindungan Privasi
            </h3>
            <p className="text-xs text-slate-400">
              Kebijakan hashing dan perlindungan kredensial akun
            </p>
          </div>
        </div>

        <div className="space-y-3 text-xs text-slate-300">
          <div className="p-4 rounded-2xl bg-slate-950/70 border border-slate-800/80 space-y-2">
            <div className="flex items-center gap-2 text-indigo-400 font-medium">
              <Key className="w-4 h-4" />
              <span>Token Sesi Saat Ini</span>
            </div>
            <p className="font-mono text-slate-400 break-all">{maskedToken}</p>
            <p className="text-[11px] text-slate-500">
              Token tidak pernah diekspos ke log atau query URL publik.
            </p>
          </div>

          <div className="p-4 rounded-2xl bg-slate-950/70 border border-slate-800/80 space-y-1">
            <div className="flex items-center gap-2 text-amber-400 font-medium">
              <Info className="w-4 h-4" />
              <span>Catatan Otorisasi & Perubahan Kredensial</span>
            </div>
            <p className="text-slate-400 text-xs leading-relaxed">
              Sesuai dengan spesifikasi REQ.md Seksi 17, Kusuri Dashboard beroperasi mengikuti kontrak backend yang aktif. Seluruh password di-hash secara aman menggunakan Argon2 di database PostgreSQL server.
            </p>
          </div>
        </div>

        <div className="pt-4 border-t border-slate-800 flex justify-end">
          <button
            onClick={logout}
            className="flex items-center gap-2 px-5 py-2.5 bg-rose-600/10 hover:bg-rose-600 text-rose-400 hover:text-white border border-rose-600/30 font-medium text-xs rounded-xl transition cursor-pointer"
          >
            <LogOut className="w-4 h-4" />
            <span>Keluar dari Akun (Logout)</span>
          </button>
        </div>
      </div>
    </div>
  );
};
