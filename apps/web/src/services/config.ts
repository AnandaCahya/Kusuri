// Configuration utility for Backend REST & WebSocket URLs

const STORAGE_API_KEY = "kusuri_api_base_url";

export function getApiBaseUrl(): string {
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem(STORAGE_API_KEY);
    if (saved && saved.trim()) {
      return saved.trim().replace(/\/+$/, "");
    }
  }

  // If in browser and on a non-dev port, can use origin, otherwise default to local Rust backend port 8080
  if (typeof window !== "undefined" && window.location.port !== "3000" && window.location.port !== "5173") {
    // If running in container or proxy behind reverse proxy
    if (window.location.origin && !window.location.origin.includes(":3000")) {
      return window.location.origin;
    }
  }

  return "http://localhost:8080";
}

export function setApiBaseUrl(url: string): void {
  if (typeof window !== "undefined") {
    if (!url || !url.trim()) {
      localStorage.removeItem(STORAGE_API_KEY);
    } else {
      localStorage.setItem(STORAGE_API_KEY, url.trim().replace(/\/+$/, ""));
    }
  }
}

export function getWsBaseUrl(): string {
  const httpUrl = getApiBaseUrl();
  try {
    const parsed = new URL(httpUrl);
    const protocol = parsed.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${parsed.host}/ws`;
  } catch {
    return httpUrl.replace(/^http/, "ws") + "/ws";
  }
}
