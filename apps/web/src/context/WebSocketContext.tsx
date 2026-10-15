import React, { createContext, useContext, useEffect, useState } from "react";
import { wsClient } from "../services/websocket";
import { useAuth } from "./AuthContext";
import { useToast } from "./ToastContext";
import type { WsConnectionStatus, WsServerEvent } from "../types";

interface WebSocketContextType {
  status: WsConnectionStatus;
  reconnect: () => void;
  send: (data: any) => boolean;
  lastEvent: WsServerEvent | null;
}

const WebSocketContext = createContext<WebSocketContextType | undefined>(undefined);

export const WebSocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { token, isAuthenticated } = useAuth();
  const { addToast } = useToast();
  const [status, setStatus] = useState<WsConnectionStatus>("disconnected");
  const [lastEvent, setLastEvent] = useState<WsServerEvent | null>(null);

  useEffect(() => {
    const unsubStatus = wsClient.onStatusChange((newStatus) => {
      setStatus(newStatus);
    });

    const unsubMsg = wsClient.onMessage((event) => {
      setLastEvent(event);

      // Handle global real-time notifications
      switch (event.type) {
        case "device_connected":
          addToast(
            `Perangkat ${event.payload.name} (${event.payload.device_type}) kini online.`,
            "info",
            "Perangkat Terhubung"
          );
          break;
        case "device_disconnected":
          addToast(
            `Perangkat telah terputus dari jaringan.`,
            "info",
            "Perangkat Offline"
          );
          break;
        case "clipboard_updated":
          addToast(
            `Riwayat clipboard diperbarui dari ${event.payload.source_device_name || "perangkat lain"}.`,
            "success",
            "Clipboard Sync"
          );
          break;
        case "file_available":
          addToast(
            `Berkas baru siap diunduh: ${event.payload.original_filename}`,
            "success",
            "Transfer Berkas"
          );
          break;
        case "command_result_received":
          addToast(
            `Hasil eksekusi remote command: ${event.payload.status}`,
            event.payload.status === "completed" ? "success" : "error",
            "Remote Command"
          );
          break;
        case "error":
          addToast(event.payload.message, "error", "WebSocket Error");
          break;
      }
    });

    if (isAuthenticated && token) {
      wsClient.connect();
    } else {
      wsClient.disconnect();
    }

    return () => {
      unsubStatus();
      unsubMsg();
      wsClient.disconnect();
    };
  }, [isAuthenticated, token, addToast]);

  const reconnect = () => {
    wsClient.disconnect();
    setTimeout(() => {
      wsClient.connect();
    }, 200);
  };

  const send = (data: any) => {
    return wsClient.send(data);
  };

  return (
    <WebSocketContext.Provider value={{ status, reconnect, send, lastEvent }}>
      {children}
    </WebSocketContext.Provider>
  );
};

export function useWebSocket(): WebSocketContextType {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error("useWebSocket must be used within a WebSocketProvider");
  }
  return context;
}
