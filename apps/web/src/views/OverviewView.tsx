import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { StatusBadge, DeviceTypeBadge, CommandStatusBadge } from "../components/common/Badge";
import { CardSkeleton } from "../components/common/Skeleton";
import type { Device, ClipboardItem, SharedFile, RemoteCommand } from "../types";
import {
  Smartphone,
  ClipboardList,
  FolderOpen,
  Terminal,
  QrCode,
  ArrowRight,
  Wifi,
  Clock,
  HardDrive,
  Copy,
  Check,
} from "lucide-react";
import { useToast } from "../context/ToastContext";

interface OverviewViewProps {
  onNavigate: (tab: string) => void;
}

export const OverviewView: React.FC<OverviewViewProps> = ({ onNavigate }) => {
  const { status, lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [devices, setDevices] = useState<Device[]>([]);
  const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>([]);
  const [files, setFiles] = useState<SharedFile[]>([]);
  const [commands, setCommands] = useState<RemoteCommand[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      const [devs, clips, fList, cmds] = await Promise.allSettled([
        api.devices.list(),
        api.clipboard.list(),
        api.files.list(),
        api.commands.list(),
      ]);

      if (devs.status === "fulfilled") setDevices(devs.value);
      if (clips.status === "fulfilled") setClipboardItems(clips.value);
      if (fList.status === "fulfilled") setFiles(fList.value);
      if (cmds.status === "fulfilled") setCommands(cmds.value);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  // Refresh when relevant WebSocket events arrive
  useEffect(() => {
    if (!lastEvent) return;
    if (
      lastEvent.type === "device_connected" ||
      lastEvent.type === "device_disconnected" ||
      lastEvent.type === "clipboard_updated" ||
      lastEvent.type === "file_available" ||
      lastEvent.type === "command_result_received"
    ) {
      fetchData();
    }
  }, [lastEvent]);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    addToast("Teks disalin ke clipboard", "success");
    setTimeout(() => setCopiedId(null), 2000);
  };

  const onlineDevicesCount = devices.filter((d) => d.is_online).length;
  const offlineDevicesCount = devices.length - onlineDevicesCount;

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-200">
      {/* Top Banner / Quick Actions */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-indigo-900/60 via-slate-900 to-indigo-950/40 border border-indigo-500/20 p-6 sm:p-8">
        <div className="relative z-10 max-w-2xl">
          <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 mb-3">
            <Wifi className="w-3.5 h-3.5" />
            Control Plane Siap
          </span>
          <h2 className="text-2xl sm:text-3xl font-bold text-white tracking-tight">
            Selamat datang di Kusuri Dashboard
          </h2>
          <p className="text-slate-300 text-sm mt-2 leading-relaxed">
            Kelola sinkronisasi otomatis antar ponsel Android, laptop Linux, dan komputer Windows Anda dari satu panel kendali terpusat.
          </p>

          <div className="flex flex-wrap gap-3 mt-6">
            <button
              onClick={() => onNavigate("pair")}
              className="flex items-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-sm rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer"
            >
              <QrCode className="w-4 h-4" />
              <span>Pair Perangkat Baru</span>
            </button>
            <button
              onClick={() => onNavigate("clipboard")}
              className="flex items-center gap-2 px-4 py-2.5 bg-slate-800/80 hover:bg-slate-700 text-slate-200 font-medium text-sm rounded-xl border border-slate-700 transition cursor-pointer"
            >
              <ClipboardList className="w-4 h-4" />
              <span>Kirim Clipboard</span>
            </button>
            <button
              onClick={() => onNavigate("files")}
              className="flex items-center gap-2 px-4 py-2.5 bg-slate-800/80 hover:bg-slate-700 text-slate-200 font-medium text-sm rounded-xl border border-slate-700 transition cursor-pointer"
            >
              <FolderOpen className="w-4 h-4" />
              <span>Upload Berkas</span>
            </button>
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Devices Stat */}
        <div
          onClick={() => onNavigate("devices")}
          className="p-5 rounded-2xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition cursor-pointer group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-slate-400">Total Perangkat</span>
            <div className="p-2.5 rounded-xl bg-indigo-600/10 text-indigo-400 group-hover:scale-110 transition">
              <Smartphone className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{devices.length}</span>
            <span className="text-xs text-slate-500">terdaftar</span>
          </div>
          <div className="mt-3 flex items-center gap-2 text-xs">
            <span className="text-emerald-400 font-medium">{onlineDevicesCount} Online</span>
            <span className="text-slate-600">•</span>
            <span className="text-slate-400">{offlineDevicesCount} Offline</span>
          </div>
        </div>

        {/* Clipboard Stat */}
        <div
          onClick={() => onNavigate("clipboard")}
          className="p-5 rounded-2xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition cursor-pointer group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-slate-400">Riwayat Clipboard</span>
            <div className="p-2.5 rounded-xl bg-violet-600/10 text-violet-400 group-hover:scale-110 transition">
              <ClipboardList className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{clipboardItems.length}</span>
            <span className="text-xs text-slate-500">item tersimpan</span>
          </div>
          <p className="mt-3 text-xs text-slate-400 truncate">
            {clipboardItems[0] ? `Terbaru: "${clipboardItems[0].content}"` : "Belum ada clipboard"}
          </p>
        </div>

        {/* Shared Files Stat */}
        <div
          onClick={() => onNavigate("files")}
          className="p-5 rounded-2xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition cursor-pointer group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-slate-400">Berkas Bersama</span>
            <div className="p-2.5 rounded-xl bg-sky-600/10 text-sky-400 group-hover:scale-110 transition">
              <FolderOpen className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{files.length}</span>
            <span className="text-xs text-slate-500">berkas</span>
          </div>
          <p className="mt-3 text-xs text-slate-400 truncate">
            {files[0] ? `Terakhir: ${files[0].original_filename}` : "Belum ada berkas"}
          </p>
        </div>

        {/* Remote Commands Stat */}
        <div
          onClick={() => onNavigate("commands")}
          className="p-5 rounded-2xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition cursor-pointer group"
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-slate-400">Remote Commands</span>
            <div className="p-2.5 rounded-xl bg-emerald-600/10 text-emerald-400 group-hover:scale-110 transition">
              <Terminal className="w-5 h-5" />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold text-white">{commands.length}</span>
            <span className="text-xs text-slate-500">dieksekusi</span>
          </div>
          <p className="mt-3 text-xs text-slate-400 truncate">
            {commands[0] ? `Terakhir: ${commands[0].command} (${commands[0].status})` : "Belum ada perintah"}
          </p>
        </div>
      </div>

      {/* Two Columns: Recent Devices & Recent Clipboard */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Devices Column */}
        <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold text-slate-200 flex items-center gap-2">
              <Smartphone className="w-4 h-4 text-indigo-400" />
              Perangkat Terhubung
            </h3>
            <button
              onClick={() => onNavigate("devices")}
              className="text-xs text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 cursor-pointer"
            >
              Lihat Semua <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>

          {devices.length === 0 ? (
            <div className="text-center py-10 text-slate-500 text-sm">
              Belum ada perangkat terhubung. Buka tab Pair Device untuk menghubungkan Android/PC Anda.
            </div>
          ) : (
            <div className="space-y-3">
              {devices.slice(0, 4).map((device) => (
                <div
                  key={device.id}
                  onClick={() => onNavigate("devices")}
                  className="flex items-center justify-between p-3.5 rounded-2xl bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 transition cursor-pointer"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="p-2 rounded-xl bg-slate-800 text-slate-300">
                      <Smartphone className="w-4 h-4" />
                    </div>
                    <div className="min-w-0">
                      <h4 className="text-sm font-medium text-slate-200 truncate">
                        {device.marketing_name || device.name}
                      </h4>
                      <p className="text-xs text-slate-500 truncate">
                        {device.chipset || device.device_type}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <StatusBadge isOnline={device.is_online} />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Clipboard Snippet Column */}
        <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold text-slate-200 flex items-center gap-2">
              <ClipboardList className="w-4 h-4 text-violet-400" />
              Riwayat Clipboard Terbaru
            </h3>
            <button
              onClick={() => onNavigate("clipboard")}
              className="text-xs text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 cursor-pointer"
            >
              Lihat Semua <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>

          {clipboardItems.length === 0 ? (
            <div className="text-center py-10 text-slate-500 text-sm">
              Belum ada riwayat clipboard. Salin teks dari perangkat terhubung atau masukkan langsung.
            </div>
          ) : (
            <div className="space-y-3">
              {clipboardItems.slice(0, 4).map((item) => (
                <div
                  key={item.id}
                  className="p-3.5 rounded-2xl bg-slate-950/60 border border-slate-800/80 flex items-start justify-between gap-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm text-slate-200 font-mono line-clamp-2 break-all">
                      {item.content}
                    </p>
                    <div className="flex items-center gap-2 mt-2 text-[11px] text-slate-500">
                      <span>{item.source_device_name || "Web"}</span>
                      <span>•</span>
                      <span>{new Date(item.created_at).toLocaleTimeString()}</span>
                    </div>
                  </div>
                  <button
                    onClick={() => copyToClipboard(item.content, item.id)}
                    title="Salin"
                    className="p-2 text-slate-400 hover:text-white bg-slate-800/80 hover:bg-slate-700 rounded-xl transition cursor-pointer shrink-0"
                  >
                    {copiedId === item.id ? (
                      <Check className="w-4 h-4 text-emerald-400" />
                    ) : (
                      <Copy className="w-4 h-4" />
                    )}
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
