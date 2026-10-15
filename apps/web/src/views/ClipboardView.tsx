import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import { ConfirmDialog } from "../components/common/ConfirmDialog";
import { EmptyState } from "../components/common/EmptyState";
import { CardSkeleton } from "../components/common/Skeleton";
import type { ClipboardItem } from "../types";
import {
  ClipboardList,
  Copy,
  Check,
  Trash2,
  Send,
  Loader2,
  Clock,
  Smartphone,
  Globe,
  FileText,
} from "lucide-react";

export const ClipboardView: React.FC = () => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [newContent, setNewContent] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Delete state
  const [deleteTarget, setDeleteTarget] = useState<ClipboardItem | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const fetchItems = async () => {
    try {
      const data = await api.clipboard.list();
      setItems(data);
    } catch (err: any) {
      addToast(err.message || "Gagal memuat riwayat clipboard", "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchItems();
  }, []);

  // Listen to WebSocket clipboard_updated event
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type === "clipboard_updated") {
      fetchItems();
    }
  }, [lastEvent]);

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newContent.trim()) return;

    setIsSending(true);
    try {
      await api.clipboard.create(newContent.trim());
      addToast("Teks berhasil disinkronkan ke seluruh perangkat!", "success");
      setNewContent("");
      fetchItems();
    } catch (err: any) {
      addToast(err.message || "Gagal mengirim teks clipboard", "error");
    } finally {
      setIsSending(false);
    }
  };

  const handleCopy = (content: string, id: string) => {
    navigator.clipboard.writeText(content);
    setCopiedId(id);
    addToast("Teks disalin ke clipboard lokal", "success");
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;

    setIsDeleting(true);
    try {
      await api.clipboard.delete(deleteTarget.id);
      addToast("Item clipboard berhasil dihapus", "success");
      setDeleteTarget(null);
      fetchItems();
    } catch (err: any) {
      addToast(err.message || "Gagal menghapus item clipboard", "error");
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <div className="space-y-6 max-w-4xl mx-auto animate-in fade-in duration-200">
      {/* Send New Clipboard Card */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-md">
        <h3 className="text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
          <Send className="w-4 h-4 text-indigo-400" />
          Kirim Teks / Tautan ke Seluruh Perangkat
        </h3>

        <form onSubmit={handleSend} className="space-y-3">
          <textarea
            value={newContent}
            onChange={(e) => setNewContent(e.target.value)}
            placeholder="Tuliskan teks, catatan, kode, atau tautan yang ingin disinkronkan..."
            rows={3}
            className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-2xl p-4 text-sm text-slate-100 placeholder:text-slate-600 focus:outline-none transition resize-y font-mono"
          />

          <div className="flex items-center justify-between">
            <span className="text-[11px] text-slate-500">
              Otomatis tersinkronisasi instan via WebSocket ke Android & Desktop
            </span>

            <button
              type="submit"
              disabled={isSending || !newContent.trim()}
              className="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium text-xs rounded-xl shadow-lg shadow-indigo-950 transition cursor-pointer"
            >
              {isSending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Send className="w-4 h-4" />
              )}
              <span>Kirim ke Perangkat</span>
            </button>
          </div>
        </form>
      </div>

      {/* Clipboard History List */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-slate-200 flex items-center gap-2">
            <ClipboardList className="w-4 h-4 text-violet-400" />
            Riwayat Sinkronisasi Clipboard ({items.length})
          </h3>
        </div>

        {isLoading ? (
          <div className="space-y-3">
            <CardSkeleton />
            <CardSkeleton />
            <CardSkeleton />
          </div>
        ) : items.length === 0 ? (
          <EmptyState
            icon={ClipboardList}
            title="Belum Ada Riwayat Clipboard"
            description="Teks yang disalin di ponsel Android, Windows, atau web ini akan otomatis muncul di sini."
          />
        ) : (
          <div className="space-y-3">
            {items.map((item) => (
              <div
                key={item.id}
                className="bg-slate-900 border border-slate-800 rounded-2xl p-4 hover:border-slate-700 transition flex flex-col sm:flex-row items-start justify-between gap-4 group"
              >
                <div className="min-w-0 flex-1 space-y-2">
                  <div className="flex items-center gap-2 text-xs text-slate-400">
                    <span className="flex items-center gap-1 font-medium text-slate-300">
                      <Smartphone className="w-3.5 h-3.5 text-indigo-400" />
                      {item.source_device_name || "Web Client"}
                    </span>
                    <span>•</span>
                    <span className="flex items-center gap-1 text-[11px] text-slate-500">
                      <Clock className="w-3 h-3" />
                      {new Date(item.created_at).toLocaleString()}
                    </span>
                    <span className="px-1.5 py-0.5 rounded text-[10px] bg-slate-800 text-slate-400">
                      {item.content_type}
                    </span>
                  </div>

                  <div className="p-3 bg-slate-950 rounded-xl border border-slate-800/80 font-mono text-xs text-slate-200 break-words whitespace-pre-wrap select-all max-h-48 overflow-y-auto">
                    {item.content}
                  </div>
                </div>

                {/* Actions */}
                <div className="flex sm:flex-col items-center gap-1 shrink-0 self-end sm:self-start">
                  <button
                    onClick={() => handleCopy(item.content, item.id)}
                    title="Salin teks"
                    className="p-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition cursor-pointer"
                  >
                    {copiedId === item.id ? (
                      <Check className="w-4 h-4 text-emerald-400" />
                    ) : (
                      <Copy className="w-4 h-4" />
                    )}
                  </button>

                  <button
                    onClick={() => setDeleteTarget(item)}
                    title="Hapus dari riwayat"
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
        title="Hapus Riwayat Clipboard?"
        message="Item ini akan dihapus dari riwayat database Kusuri secara permanen."
        confirmLabel="Hapus"
      />
    </div>
  );
};
