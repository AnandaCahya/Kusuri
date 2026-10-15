import React, { useEffect, useState } from "react";
import { api } from "../services/api";
import { useWebSocket } from "../context/WebSocketContext";
import { useToast } from "../context/ToastContext";
import type { PairingCodeResponse } from "../types";
import {
  QrCode,
  Clock,
  ShieldAlert,
  Smartphone,
  Laptop,
  CheckCircle2,
  RefreshCw,
  Loader2,
  Info,
  Copy,
  Check,
} from "lucide-react";

interface PairDeviceViewProps {
  onNavigate: (tab: string) => void;
}

export const PairDeviceView: React.FC<PairDeviceViewProps> = ({ onNavigate }) => {
  const { lastEvent } = useWebSocket();
  const { addToast } = useToast();

  const [pairingData, setPairingData] = useState<PairingCodeResponse | null>(null);
  const [secondsLeft, setSecondsLeft] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [pairedSuccessDevice, setPairedSuccessDevice] = useState<string | null>(null);
  const [isCopied, setIsCopied] = useState<boolean>(false);

  // Generate pairing code
  const generateCode = async () => {
    setIsLoading(true);
    setPairedSuccessDevice(null);
    setIsCopied(false);
    try {
      const data = await api.devices.createPairingCode();
      setPairingData(data);
      setSecondsLeft(data.expires_in_seconds || 300);
    } catch (err: any) {
      addToast(err.message || "Gagal membuat kode pairing", "error");
    } finally {
      setIsLoading(false);
    }
  };

  // Timer countdown
  useEffect(() => {
    if (secondsLeft <= 0) return;
    const interval = setInterval(() => {
      setSecondsLeft((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [secondsLeft]);

  // Listen to WebSocket device connected event
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type === "device_connected") {
      setPairedSuccessDevice(lastEvent.payload.name);
      addToast(
        `Perangkat "${lastEvent.payload.name}" berhasil dipasangkan!`,
        "success",
        "Pairing Berhasil"
      );
    }
  }, [lastEvent, addToast]);

  const copyCode = () => {
    if (!pairingData?.code) return;
    navigator.clipboard.writeText(pairingData.code);
    setIsCopied(true);
    addToast("Kode pairing disalin", "success");
    setTimeout(() => setIsCopied(false), 2000);
  };

  const formatTime = (secs: number) => {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  };

  return (
    <div className="max-w-3xl mx-auto space-y-8 animate-in fade-in duration-200">
      <div className="text-center max-w-lg mx-auto">
        <h2 className="text-2xl font-bold text-white tracking-tight">
          Pairing Perangkat Baru
        </h2>
        <p className="text-sm text-slate-400 mt-2">
          Hasilkan kode pairing 6-digit untuk menghubungkan aplikasi client Kusuri di ponsel Android atau PC Anda secara aman tanpa menyimpan password akun.
        </p>
      </div>

      {pairedSuccessDevice && (
        <div className="p-5 rounded-2xl bg-emerald-950/60 border border-emerald-600/50 flex items-center justify-between gap-4 animate-in zoom-in-95">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="w-6 h-6 text-emerald-400 shrink-0" />
            <div>
              <h4 className="font-semibold text-emerald-100 text-sm">
                Pairing Sukses!
              </h4>
              <p className="text-xs text-emerald-300">
                Perangkat <span className="font-bold">{pairedSuccessDevice}</span> telah terdaftar dan terhubung.
              </p>
            </div>
          </div>
          <button
            onClick={() => onNavigate("devices")}
            className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl text-xs font-medium cursor-pointer transition"
          >
            Buka Perangkat
          </button>
        </div>
      )}

      {/* Main Pairing Card */}
      <div className="bg-slate-900 border border-slate-800 rounded-3xl p-8 shadow-xl text-center space-y-6">
        {pairingData && secondsLeft > 0 ? (
          <div className="space-y-6">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20">
              <Clock className="w-3.5 h-3.5" />
              <span>Berlaku hingga: {formatTime(secondsLeft)}</span>
            </div>

            {/* Big 6-Digit Display */}
            <div className="flex items-center justify-center gap-3">
              <div className="px-8 py-5 bg-slate-950 border-2 border-indigo-500/40 rounded-3xl tracking-[0.3em] font-mono font-bold text-4xl sm:text-5xl text-white shadow-inner select-all">
                {pairingData.code}
              </div>
              <button
                onClick={copyCode}
                title="Salin kode"
                className="p-4 bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white rounded-2xl transition cursor-pointer"
              >
                {isCopied ? (
                  <Check className="w-6 h-6 text-emerald-400" />
                ) : (
                  <Copy className="w-6 h-6" />
                )}
              </button>
            </div>

            {/* Security notice */}
            <div className="flex items-center justify-center gap-2 text-xs text-slate-400">
              <ShieldAlert className="w-4 h-4 text-amber-400 shrink-0" />
              <span>
                Maksimal 3 kali percobaan salah. Kode otomatis hangus jika salah 3 kali atau setelah 5 menit.
              </span>
            </div>

            <button
              onClick={generateCode}
              disabled={isLoading}
              className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium text-slate-400 hover:text-white bg-slate-800/60 hover:bg-slate-800 rounded-xl transition cursor-pointer"
            >
              <RefreshCw className="w-3.5 h-3.5" />
              <span>Buat Kode Baru</span>
            </button>
          </div>
        ) : (
          <div className="py-6 space-y-4">
            <div className="w-16 h-16 rounded-2xl bg-indigo-600/10 border border-indigo-500/20 flex items-center justify-center mx-auto text-indigo-400">
              <QrCode className="w-8 h-8" />
            </div>

            {pairingData && secondsLeft === 0 && (
              <p className="text-sm text-rose-400">
                Kode pairing sebelumnya telah kedaluwarsa. Silakan buat kode baru.
              </p>
            )}

            <button
              onClick={generateCode}
              disabled={isLoading}
              className="px-6 py-3.5 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-sm rounded-2xl shadow-xl shadow-indigo-950 transition cursor-pointer inline-flex items-center gap-2 disabled:opacity-50"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>Menghasilkan Kode...</span>
                </>
              ) : (
                <>
                  <QrCode className="w-5 h-5" />
                  <span>Hasilkan Kode Pairing 6-Digit</span>
                </>
              )}
            </button>
          </div>
        )}
      </div>

      {/* Instructions */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Android Client Steps */}
        <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-3">
          <div className="flex items-center gap-2.5 text-slate-200 font-semibold text-sm">
            <Smartphone className="w-4 h-4 text-emerald-400" />
            <span>Cara Hubungkan Aplikasi Android</span>
          </div>
          <ol className="text-xs text-slate-400 space-y-2 list-decimal list-inside leading-relaxed">
            <li>Buka aplikasi <span className="text-slate-200">Kusuri</span> di perangkat Android Anda.</li>
            <li>Pilih menu <span className="text-slate-200">"Pair Device"</span>.</li>
            <li>Ketikkan 6-digit kode yang tertera di atas.</li>
            <li>Perangkat Android akan menerima <span className="font-mono text-indigo-300">device_token</span> permanen otomatis.</li>
          </ol>
        </div>

        {/* Desktop Client Steps */}
        <div className="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-3">
          <div className="flex items-center gap-2.5 text-slate-200 font-semibold text-sm">
            <Laptop className="w-4 h-4 text-sky-400" />
            <span>Cara Hubungkan Linux / Windows</span>
          </div>
          <ol className="text-xs text-slate-400 space-y-2 list-decimal list-inside leading-relaxed">
            <li>Jalankan Kusuri Desktop Client atau CLI pada komputer Anda.</li>
            <li>Eksekusi perintah pairing atau masukkan kode pada dialog connect.</li>
            <li>Kusuri Desktop otomatis menyinkronkan clipboard dan transfer berkas.</li>
            <li>Status online komputer Anda akan langsung terlihat di dashboard.</li>
          </ol>
        </div>
      </div>
    </div>
  );
};
