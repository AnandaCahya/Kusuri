import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import { CommandStatusBadge } from "../components/common/Badge";
import { Modal } from "../components/common/Modal";
import { EmptyState } from "../components/common/EmptyState";
import { CardSkeleton } from "../components/common/Skeleton";
import type { RemoteCommand, Device } from "../types";
import {
  Terminal,
  Send,
  Smartphone,
  Clock,
  CheckCircle2,
  AlertCircle,
  Code,
  Loader2,
  RefreshCw,
} from "lucide-react";

export const CommandsView: React.FC = () => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [commands, setCommands] = useState<RemoteCommand[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // New Command Form State
  const [targetDeviceId, setTargetDeviceId] = useState("");
  const [commandType, setCommandType] = useState("ping");
  const [customCommand, setCustomCommand] = useState("");
  const [rawPayload, setRawPayload] = useState("{}");
  const [isSending, setIsSending] = useState(false);

  // Selected Command for Response Detail Modal
  const [selectedCommand, setSelectedCommand] = useState<RemoteCommand | null>(null);

  const fetchData = async () => {
    try {
      const [cmdList, devList] = await Promise.all([
        api.commands.list(),
        api.devices.list().catch(() => []),
      ]);
      setCommands(cmdList);
      setDevices(devList);
      if (devList.length > 0 && !targetDeviceId) {
        const firstDevice = devList[0];
        if (firstDevice) setTargetDeviceId(firstDevice.id);
      }
    } catch (err: any) {
      addToast(err.message || "Gagal memuat riwayat command", "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  // Listen to WebSocket command_result_received event
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type === "command_result_received") {
      fetchData();
    }
  }, [lastEvent]);

  const handleSendCommand = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!targetDeviceId) {
      addToast("Pilih perangkat target terlebih dahulu", "error");
      return;
    }

    const finalCommand = commandType === "custom" ? customCommand.trim() : commandType;
    if (!finalCommand) {
      addToast("Nama perintah tidak boleh kosong", "error");
      return;
    }

    let parsedPayload: any = {};
    if (rawPayload.trim()) {
      try {
        parsedPayload = JSON.parse(rawPayload);
      } catch (e) {
        addToast("Format JSON payload tidak valid", "error");
        return;
      }
    }

    setIsSending(true);
    try {
      await api.commands.send(targetDeviceId, finalCommand, parsedPayload);
      addToast(`Perintah "${finalCommand}" berhasil dikirim!`, "success");
      fetchData();
    } catch (err: any) {
      addToast(err.message || "Gagal mengirim remote command", "error");
    } finally {
      setIsSending(false);
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in duration-200">
      {/* Send Command Box */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-md">
        <h3 className="text-sm font-semibold text-slate-200 mb-4 flex items-center gap-2">
          <Terminal className="w-4 h-4 text-emerald-400" />
          Kirim Remote Command ke Perangkat
        </h3>

        <form onSubmit={handleSendCommand} className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {/* Target device selector */}
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1.5">
                Perangkat Sasaran
              </label>
              <select
                value={targetDeviceId}
                onChange={(e) => setTargetDeviceId(e.target.value)}
                required
                className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-3.5 py-2.5 text-xs text-slate-200 focus:outline-none cursor-pointer"
              >
                {devices.length === 0 && (
                  <option value="">Tidak ada perangkat terhubung</option>
                )}
                {devices.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.marketing_name || d.name} ({d.is_online ? "Online" : "Offline"})
                  </option>
                ))}
              </select>
            </div>

            {/* Command preset */}
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1.5">
                Jenis Command
              </label>
              <select
                value={commandType}
                onChange={(e) => setCommandType(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-3.5 py-2.5 text-xs text-slate-200 focus:outline-none cursor-pointer"
              >
                <option value="ping">ping (Cek Status / Pong)</option>
                <option value="lock_screen">lock_screen (Kunci Layar)</option>
                <option value="open_url">open_url (Buka URL di Browser)</option>
                <option value="get_status">get_status (Minta Telemetri Terkini)</option>
                <option value="custom">Perintah Kustom...</option>
              </select>
            </div>
          </div>

          {commandType === "custom" && (
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1.5">
                Nama Perintah Kustom
              </label>
              <input
                type="text"
                value={customCommand}
                onChange={(e) => setCustomCommand(e.target.value)}
                placeholder="misal: sync_now, ring_phone"
                required
                className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-4 py-2 text-xs text-slate-200 focus:outline-none"
              />
            </div>
          )}

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1.5">
              Payload (JSON)
            </label>
            <textarea
              value={rawPayload}
              onChange={(e) => setRawPayload(e.target.value)}
              rows={2}
              placeholder='{"url": "https://example.com"}'
              className="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl p-3 text-xs text-slate-200 font-mono focus:outline-none resize-y"
            />
          </div>

          <div className="flex items-center justify-between pt-1">
            <span className="text-[11px] text-slate-500">
              Perintah akan dikirim secara asinkron via WebSocket jika perangkat online
            </span>

            <button
              type="submit"
              disabled={isSending || devices.length === 0}
              className="flex items-center gap-2 px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white font-medium text-xs rounded-xl shadow-lg shadow-emerald-950 transition cursor-pointer"
            >
              {isSending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Send className="w-4 h-4" />
              )}
              <span>Kirim Perintah</span>
            </button>
          </div>
        </form>
      </div>

      {/* Commands History List */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-slate-200 flex items-center gap-2">
            <Terminal className="w-4 h-4 text-emerald-400" />
            Riwayat Remote Command ({commands.length})
          </h3>
          <button
            onClick={fetchData}
            title="Refresh"
            className="p-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-400 hover:text-white transition cursor-pointer"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>

        {isLoading ? (
          <div className="space-y-3">
            <CardSkeleton />
            <CardSkeleton />
          </div>
        ) : commands.length === 0 ? (
          <EmptyState
            icon={Terminal}
            title="Belum Ada Perintah yang Dikirim"
            description="Kirim remote command untuk berinteraksi dengan perangkat target dari jarak jauh."
          />
        ) : (
          <div className="space-y-3">
            {commands.map((cmd) => (
              <div
                key={cmd.id}
                className="bg-slate-900 border border-slate-800 rounded-2xl p-4 hover:border-slate-700 transition flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 group"
              >
                <div className="space-y-1.5 min-w-0 flex-1">
                  <div className="flex items-center gap-2.5">
                    <span className="font-mono text-sm font-semibold text-slate-200">
                      {cmd.command}
                    </span>
                    <CommandStatusBadge status={cmd.status} />
                  </div>

                  <div className="flex flex-wrap items-center gap-3 text-xs text-slate-400">
                    <span className="flex items-center gap-1">
                      <Smartphone className="w-3.5 h-3.5 text-indigo-400" />
                      Target: {cmd.target_device_name || cmd.target_device_id.substring(0, 8)}
                    </span>
                    <span>•</span>
                    <span className="flex items-center gap-1 text-[11px] text-slate-500">
                      <Clock className="w-3 h-3" />
                      {new Date(cmd.created_at).toLocaleString()}
                    </span>
                  </div>
                </div>

                {/* Inspect button if has response or payload */}
                <div className="flex items-center gap-2 self-end sm:self-center">
                  <button
                    onClick={() => setSelectedCommand(cmd)}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-xs font-medium text-slate-300 transition cursor-pointer"
                  >
                    <Code className="w-3.5 h-3.5" />
                    <span>Lihat Respon</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Response Detail Modal */}
      <Modal
        isOpen={!!selectedCommand}
        onClose={() => setSelectedCommand(null)}
        title={`Detail Command: ${selectedCommand?.command}`}
        maxWidth="lg"
      >
        {selectedCommand && (
          <div className="space-y-4 text-xs">
            <div className="flex items-center justify-between p-3 rounded-xl bg-slate-950 border border-slate-800">
              <span className="text-slate-400">Status Eksekusi</span>
              <CommandStatusBadge status={selectedCommand.status} />
            </div>

            <div>
              <span className="text-slate-400 block mb-1">Payload Terkirim:</span>
              <pre className="p-3 rounded-xl bg-slate-950 border border-slate-800 text-slate-300 font-mono overflow-x-auto">
                {JSON.stringify(selectedCommand.payload, null, 2)}
              </pre>
            </div>

            <div>
              <span className="text-slate-400 block mb-1">Response dari Perangkat:</span>
              <pre className="p-3 rounded-xl bg-slate-950 border border-slate-800 text-emerald-300 font-mono overflow-x-auto">
                {selectedCommand.response
                  ? JSON.stringify(selectedCommand.response, null, 2)
                  : "Menunggu respon dari perangkat target..."}
              </pre>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
