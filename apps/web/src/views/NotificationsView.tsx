import React, { useEffect, useState, useMemo } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import { EmptyState } from "../components/common/EmptyState";
import { CardSkeleton } from "../components/common/Skeleton";
import type { DeviceNotification, Device } from "../types";
import {
  Bell,
  BellRing,
  Smartphone,
  Monitor,
  Search,
  Filter,
  RefreshCw,
  Clock,
  Layers,
  Sparkles,
  ChevronRight,
  ShieldAlert,
} from "lucide-react";

export const NotificationsView: React.FC = () => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [notifications, setNotifications] = useState<DeviceNotification[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Filters
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>("all");

  const fetchNotifications = async (showLoading = true) => {
    if (showLoading) setIsLoading(true);
    else setIsRefreshing(true);

    try {
      const [notifsData, devsData] = await Promise.all([
        api.notifications.list({
          device_id: selectedDeviceId !== "all" ? selectedDeviceId : undefined,
          search: searchQuery.trim() || undefined,
          limit: 100,
        }),
        devices.length === 0 ? api.devices.list() : Promise.resolve(devices),
      ]);

      setNotifications(notifsData);
      if (devices.length === 0) setDevices(devsData);
    } catch (err: any) {
      addToast(err.message || "Gagal memuat daftar riwayat notifikasi", "error");
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    fetchNotifications();
  }, [selectedDeviceId]);

  // Real-time live update saat notifikasi baru masuk via WebSocket
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type === "notification_ingested") {
      const newNotif = lastEvent.payload;
      setNotifications((prev) => {
        // Cek jika sudah ada
        if (prev.some((n) => n.id === newNotif.id)) return prev;
        return [newNotif, ...prev];
      });
      addToast(`Notifikasi baru dari ${newNotif.app_title || newNotif.package_or_app_name}: ${newNotif.title}`, "info");
    }
  }, [lastEvent]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    fetchNotifications();
  };

  const filteredNotifications = useMemo(() => {
    if (!searchQuery.trim()) return notifications;
    const q = searchQuery.toLowerCase();
    return notifications.filter(
      (n) =>
        n.title.toLowerCase().includes(q) ||
        n.content.toLowerCase().includes(q) ||
        (n.app_title && n.app_title.toLowerCase().includes(q)) ||
        n.package_or_app_name.toLowerCase().includes(q) ||
        (n.device_name && n.device_name.toLowerCase().includes(q))
    );
  }, [notifications, searchQuery]);

  const formatPostedAt = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.toLocaleString("id-ID", {
        day: "numeric",
        month: "short",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in duration-200">
      {/* Top Banner Info */}
      <div className="p-5 rounded-2xl bg-gradient-to-r from-indigo-950/60 via-slate-900 to-slate-950 border border-indigo-500/20 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 shadow-xl shadow-indigo-950/10">
        <div className="flex items-center gap-3.5">
          <div className="w-12 h-12 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <BellRing className="w-6 h-6 animate-pulse" />
          </div>
          <div>
            <h3 className="text-base font-semibold text-white flex items-center gap-2">
              Riwayat Notifikasi Perangkat
              <span className="px-2 py-0.5 text-[10px] font-medium rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                Live Sync 24/7
              </span>
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              Seluruh notifikasi dari smartphone Android dan PC Desktop tersimpan dan tersinkronisasi otomatis secara persisten.
            </p>
          </div>
        </div>

        <button
          onClick={() => fetchNotifications(false)}
          disabled={isRefreshing || isLoading}
          className="flex items-center gap-2 px-3.5 py-2 text-xs font-medium rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition cursor-pointer self-stretch sm:self-auto justify-center"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
          <span>Segarkan</span>
        </button>
      </div>

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 p-3.5 rounded-2xl bg-slate-900/60 border border-slate-800">
        {/* Search input */}
        <form onSubmit={handleSearchSubmit} className="flex-1 relative">
          <Search className="w-4 h-4 text-slate-500 absolute left-3.5 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Cari notifikasi berdasarkan judul, konten, atau nama aplikasi..."
            className="w-full pl-10 pr-4 py-2 bg-slate-950 border border-slate-800 rounded-xl text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition"
          />
        </form>

        {/* Device selector filter */}
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-slate-400 shrink-0 ml-1" />
          <select
            value={selectedDeviceId}
            onChange={(e) => setSelectedDeviceId(e.target.value)}
            className="bg-slate-950 border border-slate-800 text-xs text-slate-200 rounded-xl px-3 py-2 focus:outline-none focus:border-indigo-500 transition cursor-pointer"
          >
            <option value="all">Semua Perangkat</option>
            {devices.map((dev) => (
              <option key={dev.id} value={dev.id}>
                {dev.marketing_name || dev.name} ({dev.device_type})
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Notifications List */}
      {isLoading ? (
        <div className="space-y-3">
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
        </div>
      ) : filteredNotifications.length === 0 ? (
        <EmptyState
          icon={Bell}
          title="Belum Ada Notifikasi"
          description={
            searchQuery || selectedDeviceId !== "all"
              ? "Tidak ditemukan notifikasi yang cocok dengan kriteria filter."
              : "Notifikasi yang masuk di ponsel Android atau PC Anda akan otomatis tercatat dan muncul di sini secara real-time."
          }
        />
      ) : (
        <div className="space-y-3">
          <div className="flex items-center justify-between text-xs text-slate-400 px-1">
            <span>Menampilkan {filteredNotifications.length} notifikasi</span>
            <span className="flex items-center gap-1.5 text-[11px] text-slate-500">
              <Clock className="w-3.5 h-3.5" /> Diurutkan dari yang terbaru
            </span>
          </div>

          {filteredNotifications.map((notif) => {
            const isAndroid = notif.package_or_app_name.includes(".") && !notif.package_or_app_name.includes("\\");
            return (
              <div
                key={notif.id}
                className="group p-4 rounded-2xl bg-slate-900/40 hover:bg-slate-900/80 border border-slate-800/80 hover:border-slate-700 transition duration-200 shadow-sm"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-start gap-3.5 min-w-0">
                    {/* App / Device Icon badge */}
                    <div className="w-10 h-10 rounded-xl bg-slate-800 border border-slate-700/60 flex items-center justify-center text-slate-300 shrink-0 group-hover:border-indigo-500/30 transition">
                      {isAndroid ? (
                        <Smartphone className="w-5 h-5 text-emerald-400" />
                      ) : (
                        <Monitor className="w-5 h-5 text-sky-400" />
                      )}
                    </div>

                    <div className="min-w-0 flex-1">
                      {/* App header and device tag */}
                      <div className="flex flex-wrap items-center gap-2 mb-1">
                        <span className="font-semibold text-xs text-indigo-300">
                          {notif.app_title || notif.package_or_app_name}
                        </span>
                        {notif.device_name && (
                          <span className="px-2 py-0.5 rounded-full text-[10px] font-medium bg-slate-800 text-slate-300 border border-slate-700/60">
                            {notif.device_name}
                          </span>
                        )}
                        <span className="text-[10px] text-slate-500 font-mono">
                          {notif.package_or_app_name}
                        </span>
                      </div>

                      {/* Notification Title */}
                      <h4 className="font-medium text-slate-100 text-sm mb-1 leading-snug">
                        {notif.title}
                      </h4>

                      {/* Notification Content */}
                      <p className="text-xs text-slate-300 leading-relaxed whitespace-pre-wrap break-words">
                        {notif.content}
                      </p>

                      {/* Subtext if present */}
                      {notif.subtext && (
                        <p className="text-[11px] text-slate-500 mt-1 italic">
                          {notif.subtext}
                        </p>
                      )}
                    </div>
                  </div>

                  {/* Timestamp */}
                  <div className="shrink-0 text-right">
                    <span className="text-[11px] text-slate-400 block whitespace-nowrap">
                      {formatPostedAt(notif.posted_at)}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
