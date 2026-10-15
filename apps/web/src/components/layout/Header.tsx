import React from "react";
import { useAuth } from "../../context/AuthContext";
import { useWebSocket } from "../../context/WebSocketContext";
import {
  Wifi,
  WifiOff,
  RefreshCw,
  LogOut,
  User as UserIcon,
  ShieldCheck,
} from "lucide-react";

interface HeaderProps {
  currentTab: string;
  onOpenMobileMenu?: () => void;
}

export const Header: React.FC<HeaderProps> = ({ currentTab }) => {
  const { user, logout } = useAuth();
  const { status, reconnect } = useWebSocket();

  const getStatusBadge = () => {
    switch (status) {
      case "connected":
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full bg-emerald-950/70 text-emerald-400 border border-emerald-800/60">
            <Wifi className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Real-time Aktif</span>
          </span>
        );
      case "connecting":
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full bg-amber-950/70 text-amber-400 border border-amber-800/60">
            <RefreshCw className="w-3.5 h-3.5 animate-spin" />
            <span className="hidden sm:inline">Menghubungkan...</span>
          </span>
        );
      case "disconnected":
      case "error":
      default:
        return (
          <button
            onClick={reconnect}
            title="Klik untuk menghubungkan ulang WebSocket"
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full bg-rose-950/70 text-rose-400 border border-rose-800/60 hover:bg-rose-900 transition cursor-pointer"
          >
            <WifiOff className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Terputus (Hubungkan)</span>
          </button>
        );
    }
  };

  const getTabTitle = () => {
    switch (currentTab) {
      case "overview":
        return "Dashboard Ringkasan";
      case "devices":
        return "Manajemen Perangkat";
      case "pair":
        return "Pair Perangkat Baru";
      case "clipboard":
        return "Riwayat Clipboard";
      case "files":
        return "Manajemen Berkas";
      case "commands":
        return "Remote Commands";
      case "settings":
        return "Pengaturan Akun & Sistem";
      case "admin":
        return "Instance Administration";
      default:
        return "Dashboard";
    }
  };

  return (
    <header className="h-16 border-b border-slate-800 bg-slate-900/50 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-30">
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold text-slate-100">{getTabTitle()}</h1>
      </div>

      <div className="flex items-center gap-4">
        {/* WebSocket Status */}
        {getStatusBadge()}

        {/* User Profile & Logout */}
        <div className="flex items-center gap-3 pl-3 border-l border-slate-800">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-full bg-indigo-600/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
              <UserIcon className="w-4 h-4" />
            </div>
            <div className="hidden md:block text-left">
              <p className="text-xs font-medium text-slate-200 leading-tight">
                {user?.username}
              </p>
              <p className="text-[10px] text-slate-500">Kusuri User</p>
            </div>
          </div>

          <button
            onClick={logout}
            title="Keluar dari akun"
            className="p-2 text-slate-400 hover:text-rose-400 hover:bg-slate-800/60 rounded-xl transition cursor-pointer"
          >
            <LogOut className="w-4 h-4" />
          </button>
        </div>
      </div>
    </header>
  );
};
