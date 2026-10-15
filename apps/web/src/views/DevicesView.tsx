import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import { StatusBadge, DeviceTypeBadge } from "../components/common/Badge";
import { ConfirmDialog } from "../components/common/ConfirmDialog";
import { Modal } from "../components/common/Modal";
import { EmptyState } from "../components/common/EmptyState";
import { CardSkeleton } from "../components/common/Skeleton";
import { DeviceDetailModal } from "./DeviceDetailModal";
import type { Device } from "../types";
import {
  Smartphone,
  Search,
  Edit2,
  Trash2,
  Info,
  Clock,
  QrCode,
  Loader2,
  RefreshCw,
  Cpu,
} from "lucide-react";

interface DevicesViewProps {
  onNavigate: (tab: string) => void;
}

export const DevicesView: React.FC<DevicesViewProps> = ({ onNavigate }) => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [devices, setDevices] = useState<Device[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | "online" | "offline">("all");

  // Selected device for detail modal
  const [detailDevice, setDetailDevice] = useState<Device | null>(null);

  // Rename state
  const [renameDevice, setRenameDevice] = useState<Device | null>(null);
  const [newName, setNewName] = useState("");
  const [isRenaming, setIsRenaming] = useState(false);

  // Delete state
  const [deleteTarget, setDeleteTarget] = useState<Device | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const fetchDevices = async () => {
    try {
      const data = await api.devices.list();
      setDevices(data);
    } catch (err: any) {
      addToast(err.message || "Gagal memuat daftar perangkat", "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDevices();
  }, []);

  // Listen to WebSocket events for real-time updates
  useEffect(() => {
    if (!lastEvent) return;
    if (
      lastEvent.type === "device_connected" ||
      lastEvent.type === "device_disconnected" ||
      lastEvent.type === "device_status_updated" ||
      lastEvent.type === "device_specs_updated"
    ) {
      fetchDevices();
    }
  }, [lastEvent]);

  const handleRename = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!renameDevice || !newName.trim()) return;

    setIsRenaming(true);
    try {
      await api.devices.update(renameDevice.id, newName.trim());
      addToast(`Nama perangkat berhasil diubah menjadi "${newName.trim()}".`, "success");
      setRenameDevice(null);
      fetchDevices();
    } catch (err: any) {
      addToast(err.message || "Gagal mengubah nama perangkat", "error");
    } finally {
      setIsRenaming(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;

    setIsDeleting(true);
    try {
      await api.devices.delete(deleteTarget.id);
      addToast(`Perangkat "${deleteTarget.name}" berhasil dihapus.`, "success");
      setDeleteTarget(null);
      fetchDevices();
    } catch (err: any) {
      addToast(err.message || "Gagal menghapus perangkat", "error");
    } finally {
      setIsDeleting(false);
    }
  };

  const filteredDevices = devices.filter((dev) => {
    const matchesSearch =
      (dev.name && dev.name.toLowerCase().includes(searchQuery.toLowerCase())) ||
      (dev.marketing_name && dev.marketing_name.toLowerCase().includes(searchQuery.toLowerCase())) ||
      (dev.chipset && dev.chipset.toLowerCase().includes(searchQuery.toLowerCase())) ||
      (dev.raw_model && dev.raw_model.toLowerCase().includes(searchQuery.toLowerCase()));

    if (statusFilter === "online") return matchesSearch && dev.is_online;
    if (statusFilter === "offline") return matchesSearch && !dev.is_online;
    return matchesSearch;
  });

  return (
    <div className="space-y-6 animate-in fade-in duration-200">
      {/* Top action & filter bar */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3 flex-1 max-w-md">
          <div className="relative flex-1">
            <Search className="w-4 h-4 text-slate-500 absolute left-3.5 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Cari perangkat, model, atau chipset..."
              className="w-full bg-slate-900 border border-slate-800 focus:border-indigo-500 rounded-xl pl-10 pr-4 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:outline-none transition"
            />
          </div>

          <div className="flex bg-slate-900 border border-slate-800 rounded-xl p-1 shrink-0">
            <button
              onClick={() => setStatusFilter("all")}
              className={`px-3 py-1 text-xs font-medium rounded-lg transition cursor-pointer ${
                statusFilter === "all"
                  ? "bg-indigo-600 text-white"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              Semua
            </button>
            <button
              onClick={() => setStatusFilter("online")}
              className={`px-3 py-1 text-xs font-medium rounded-lg transition cursor-pointer ${
                statusFilter === "online"
                  ? "bg-indigo-600 text-white"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              Online
            </button>
            <button
              onClick={() => setStatusFilter("offline")}
              className={`px-3 py-1 text-xs font-medium rounded-lg transition cursor-pointer ${
                statusFilter === "offline"
                  ? "bg-indigo-600 text-white"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              Offline
            </button>
          </div>
        </div>

        <button
          onClick={() => onNavigate("pair")}
          className="flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-sm rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer shrink-0"
        >
          <QrCode className="w-4 h-4" />
          <span>Pair Perangkat Baru</span>
        </button>
      </div>

      {/* Device Grid */}
      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
        </div>
      ) : filteredDevices.length === 0 ? (
        <EmptyState
          icon={Smartphone}
          title={searchQuery ? "Perangkat Tidak Ditemukan" : "Belum Ada Perangkat"}
          description={
            searchQuery
              ? "Tidak ada perangkat yang cocok dengan kriteria pencarian Anda."
              : "Hubungkan ponsel Android, komputer Windows, atau laptop Linux Anda dengan membuat kode pairing."
          }
          actionLabel={searchQuery ? undefined : "Buat Kode Pairing"}
          onAction={searchQuery ? undefined : () => onNavigate("pair")}
        />
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredDevices.map((device) => (
            <div
              key={device.id}
              className="bg-slate-900 border border-slate-800 rounded-2xl p-5 hover:border-slate-700 transition flex flex-col justify-between group shadow-sm hover:shadow-md"
            >
              <div>
                <div className="flex items-start justify-between gap-3 mb-3">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="p-2.5 rounded-xl bg-slate-950 border border-slate-800 text-indigo-400 shrink-0">
                      <Smartphone className="w-5 h-5" />
                    </div>
                    <div className="min-w-0">
                      <h4 className="font-semibold text-slate-100 text-sm truncate">
                        {device.marketing_name || device.name}
                      </h4>
                      <p className="text-xs text-slate-400 truncate">
                        {device.name !== device.marketing_name ? `${device.name} • ` : ""}
                        {device.retail_branding || device.raw_model || device.device_type}
                      </p>
                    </div>
                  </div>
                  <StatusBadge isOnline={device.is_online} />
                </div>

                {device.chipset && (
                  <div className="mb-4 p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80 text-xs text-slate-300 flex items-center gap-2">
                    <Cpu className="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                    <span className="truncate">{device.chipset}</span>
                  </div>
                )}

                <div className="flex items-center justify-between text-xs text-slate-500 mb-4 pt-2 border-t border-slate-800/80">
                  <DeviceTypeBadge type={device.device_type} />
                  <span className="flex items-center gap-1">
                    <Clock className="w-3.5 h-3.5" />
                    {device.last_seen_at
                      ? new Date(device.last_seen_at).toLocaleDateString()
                      : "Baru saja"}
                  </span>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-between gap-2 pt-3 border-t border-slate-800">
                <button
                  onClick={() => setDetailDevice(device)}
                  className="flex-1 flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-200 text-xs font-medium transition cursor-pointer"
                >
                  <Info className="w-3.5 h-3.5" />
                  <span>Detail & Spek</span>
                </button>

                <button
                  onClick={() => {
                    setRenameDevice(device);
                    setNewName(device.name);
                  }}
                  title="Ubah nama"
                  className="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-400 hover:text-white transition cursor-pointer"
                >
                  <Edit2 className="w-3.5 h-3.5" />
                </button>

                <button
                  onClick={() => setDeleteTarget(device)}
                  title="Hapus perangkat"
                  className="p-2 rounded-xl bg-slate-800/80 hover:bg-rose-950 text-slate-400 hover:text-rose-400 transition cursor-pointer"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Device Detail & Specs Modal */}
      <DeviceDetailModal
        device={detailDevice}
        isOpen={!!detailDevice}
        onClose={() => setDetailDevice(null)}
      />

      {/* Rename Device Modal */}
      <Modal
        isOpen={!!renameDevice}
        onClose={() => setRenameDevice(null)}
        title="Ubah Nama Perangkat"
        maxWidth="sm"
      >
        <form onSubmit={handleRename} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Nama Tampilan Baru
            </label>
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              required
              className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-4 py-2.5 text-sm text-slate-100 placeholder:text-slate-600 focus:outline-none transition"
            />
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setRenameDevice(null)}
              className="px-4 py-2 text-xs font-medium text-slate-300 bg-slate-800 hover:bg-slate-700 rounded-xl transition cursor-pointer"
            >
              Batal
            </button>
            <button
              type="submit"
              disabled={isRenaming}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-xl transition cursor-pointer disabled:opacity-50"
            >
              {isRenaming && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              <span>Simpan Nama</span>
            </button>
          </div>
        </form>
      </Modal>

      {/* Delete Device Confirm Dialog */}
      <ConfirmDialog
        isOpen={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
        isLoading={isDeleting}
        title="Hapus Perangkat?"
        message={`Apakah Anda yakin ingin menghapus "${deleteTarget?.name}" dari akun Anda? Perangkat ini akan dicabut aksesnya dan harus dipairing ulang untuk dapat tersambung kembali.`}
        confirmLabel="Hapus Perangkat"
        isDestructive={true}
      />
    </div>
  );
};
