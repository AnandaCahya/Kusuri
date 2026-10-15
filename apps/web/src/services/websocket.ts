import { getWsBaseUrl } from "./config";
import { getStoredToken } from "./api";
import type { WsConnectionStatus, WsServerEvent } from "../types";

type EventListener = (event: WsServerEvent) => void;
type StatusListener = (status: WsConnectionStatus) => void;

class WebSocketClient {
  private ws: WebSocket | null = null;
  private status: WsConnectionStatus = "disconnected";
  private eventListeners: Set<EventListener> = new Set();
  private statusListeners: Set<StatusListener> = new Set();

  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectTimer: any = null;
  private pingInterval: any = null;
  private isExplicitlyClosed = false;

  public getStatus(): WsConnectionStatus {
    return this.status;
  }

  private setStatus(newStatus: WsConnectionStatus) {
    if (this.status !== newStatus) {
      this.status = newStatus;
      this.statusListeners.forEach((listener) => {
        try {
          listener(newStatus);
        } catch (e) {
          console.error("Error in status listener:", e);
        }
      });
    }
  }

  public connect(): void {
    const token = getStoredToken();
    if (!token) {
      this.setStatus("disconnected");
      return;
    }

    if (
      this.ws &&
      (this.ws.readyState === WebSocket.OPEN ||
        this.ws.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    this.isExplicitlyClosed = false;
    this.setStatus("connecting");

    const baseUrl = getWsBaseUrl();
    const wsUrl = `${baseUrl}?token=${encodeURIComponent(token)}`;

    try {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.setStatus("connected");
        this.startHeartbeat();
      };

      this.ws.onmessage = (messageEvent) => {
        try {
          const parsed = JSON.parse(messageEvent.data) as WsServerEvent;
          this.eventListeners.forEach((listener) => {
            try {
              listener(parsed);
            } catch (e) {
              console.error("Error in WS message listener:", e);
            }
          });
        } catch (err) {
          console.warn("Failed to parse incoming WS message:", messageEvent.data, err);
        }
      };

      this.ws.onerror = (err) => {
        console.warn("WebSocket error:", err);
        this.setStatus("error");
      };

      this.ws.onclose = () => {
        this.stopHeartbeat();
        this.ws = null;
        if (!this.isExplicitlyClosed) {
          this.setStatus("disconnected");
          this.scheduleReconnect();
        } else {
          this.setStatus("disconnected");
        }
      };
    } catch (err) {
      console.error("Failed to construct WebSocket:", err);
      this.setStatus("error");
      this.scheduleReconnect();
    }
  }

  public disconnect(): void {
    this.isExplicitlyClosed = true;
    this.clearReconnectTimer();
    this.stopHeartbeat();
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        // ignore
      }
      this.ws = null;
    }
    this.setStatus("disconnected");
  }

  public send(data: any): boolean {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(typeof data === "string" ? data : JSON.stringify(data));
      return true;
    }
    return false;
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    // Send ping every 25 seconds
    this.pingInterval = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.send({ type: "ping" });
      }
    }, 25000);
  }

  private stopHeartbeat(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  private scheduleReconnect(): void {
    this.clearReconnectTimer();
    if (this.isExplicitlyClosed) return;

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.warn("Max WebSocket reconnect attempts reached. Waiting for explicit reconnect.");
      this.setStatus("error");
      return;
    }

    // Exponential backoff with jitter: min 1s, max 30s
    const baseDelay = Math.min(1000 * Math.pow(1.5, this.reconnectAttempts), 30000);
    const jitter = Math.random() * 1000;
    const delay = baseDelay + jitter;

    this.reconnectAttempts++;
    this.setStatus("connecting");

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  public onMessage(listener: EventListener): () => void {
    this.eventListeners.add(listener);
    return () => {
      this.eventListeners.delete(listener);
    };
  }

  public onStatusChange(listener: StatusListener): () => void {
    this.statusListeners.add(listener);
    // immediately call with current status
    listener(this.status);
    return () => {
      this.statusListeners.delete(listener);
    };
  }
}

export const wsClient = new WebSocketClient();
