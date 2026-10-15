import React, { useEffect, useState, useRef } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import { ConfirmDialog } from "../components/common/ConfirmDialog";
import { EmptyState } from "../components/common/EmptyState";
import { CardSkeleton } from "../components/common/Skeleton";
import type { SharedFile, Device } from "../types";
import {
  FolderOpen,
  UploadCloud,
  Download,
  Trash2,
  File,
  HardDrive,
  Clock,
  Smartphone,
  Loader2,
  Share2,
} from "lucide-react";

export const FilesView: React.FC = () => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [files, setFiles] = useState<SharedFile[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const [targetDeviceId, setTargetDeviceId] = useState<string>("");

  // Delete state
  const [deleteTarget, setDeleteTarget] = useState<SharedFile | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const fetchFiles = async () => {
    try {
      const [fList, dList] = await Promise.all([
        api.files.list(),
        api.devices.list().catch(() => []),
      ]);
      setFiles(fList);
      setDevices(dList);
    } catch (err: any) {
      addToast(err.message || "Gagal memuat daftar berkas", "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchFiles();
  }, []);

  // Listen to WebSocket file_available event
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type === "file_available") {
      fetchFiles();
    }
  }, [lastEvent]);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (!selectedFile) return;

    setIsUploading(true);
    try {
      await api.files.upload(
        selectedFile,
        targetDeviceId ? targetDeviceId : undefined
      );
      addToast(`Berkas "${selectedFile.name}" berhasil diunggah!`, "success");
      if (fileInputRef.current) fileInputRef.current.value = "";
      fetchFiles();
    } catch (err: any) {
      addToast(err.message || "Gagal mengunggah berkas", "error");
    } finally {
      setIsUploading(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;

    setIsDeleting(true);
    try {
      await api.files.delete(deleteTarget.id);
      addToast(`Berkas "${deleteTarget.original_filename}" berhasil dihapus`, "success");
      setDeleteTarget(null);
      fetchFiles();
    } catch (err: any) {
      addToast(err.message || "Gagal menghapus berkas", "error");
    } finally {
      setIsDeleting(false);
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in duration-200">
      {/* Upload Box */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-md">
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4">
          <div className="space-y-1">
            <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
              <UploadCloud className="w-4 h-4 text-sky-400" />
              Unggah & Bagikan Berkas
            </h3>
            <p className="text-xs text-slate-400">
              Kirim berkas ke perangkat tertentu atau broadcast ke semua perangkat Anda
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            {/* Target device selector */}
            <select
              value={targetDeviceId}
              onChange={(e) => setTargetDeviceId(e.target.value)}
              className="bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-slate-300 focus:outline-none focus:border-indigo-500 cursor-pointer"
            >
              <option value="">Broadcast ke Semua Perangkat</option>
              {devices.map((d) => (
                <option key={d.id} value={d.id}>
                  Kirim ke: {d.marketing_name || d.name}
                </option>
              ))}
            </select>

            {/* Hidden file input */}
            <input
              ref={fileInputRef}
              type="file"
              onChange={handleFileUpload}
              className="hidden"
            />

            <button
              onClick={() => fileInputRef.current?.click()}
              disabled={isUploading}
              className="flex items-center gap-2 px-5 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium text-xs rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer"
            >
              {isUploading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <UploadCloud className="w-4 h-4" />
              )}
              <span>{isUploading ? "Mengunggah..." : "Pilih & Kirim Berkas"}</span>
            </button>
          </div>
        </div>
      </div>

      {/* Shared Files List */}
      <div className="space-y-4">
        <h3 className="text-base font-semibold text-slate-200 flex items-center gap-2">
          <FolderOpen className="w-4 h-4 text-sky-400" />
          Daftar Berkas Tersimpan ({files.length})
        </h3>

        {isLoading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <CardSkeleton />
            <CardSkeleton />
          </div>
        ) : files.length === 0 ? (
          <EmptyState
            icon={FolderOpen}
            title="Belum Ada Berkas yang Dibagikan"
            description="Unggah foto, dokumen, atau berkas apa saja untuk ditransfer langsung antar perangkat Anda."
            actionLabel="Unggah Berkas Sekarang"
            onAction={() => fileInputRef.current?.click()}
          />
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {files.map((file) => (
              <div
                key={file.id}
                className="bg-slate-900 border border-slate-800 rounded-2xl p-4 hover:border-slate-700 transition flex items-center justify-between gap-4 group"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="p-3 rounded-xl bg-slate-950 border border-slate-800 text-sky-400 shrink-0">
                    <File className="w-6 h-6" />
                  </div>
                  <div className="min-w-0">
                    <h4 className="font-medium text-slate-200 text-sm truncate" title={file.original_filename}>
                      {file.original_filename}
                    </h4>
                    <div className="flex items-center gap-2 mt-1 text-xs text-slate-400">
                      <span>{formatFileSize(file.file_size)}</span>
                      <span>•</span>
                      <span className="flex items-center gap-1 text-[11px] text-slate-500">
                        <Clock className="w-3 h-3" />
                        {new Date(file.created_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1 shrink-0">
                  <a
                    href={api.files.getDownloadUrl(file.id)}
                    download={file.original_filename}
                    title="Unduh Berkas"
                    className="p-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition inline-flex items-center justify-center cursor-pointer"
                  >
                    <Download className="w-4 h-4" />
                  </a>

                  <button
                    onClick={() => setDeleteTarget(file)}
                    title="Hapus Berkas"
                    className="p-2 rounded-xl bg-slate-800 hover:bg-rose-950 text-slate-400 hover:text-rose-400 transition cursor-pointer"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Delete Confirm Dialog */}
      <ConfirmDialog
        isOpen={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
        isLoading={isDeleting}
        title="Hapus Berkas?"
        message={`Apakah Anda yakin ingin menghapus berkas "${deleteTarget?.original_filename}"? Berkas fisik akan dihapus dari server.`}
        confirmLabel="Hapus Berkas"
      />
    </div>
  );
};
