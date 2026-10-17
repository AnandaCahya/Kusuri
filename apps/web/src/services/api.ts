import { getApiBaseUrl } from "./config";
import type {
  AuthResponse,
  ClipboardItem,
  CreatePairingCodeRequest,
  Device,
  EnrichedDeviceSpecsResponse,
  PairingCodeResponse,
  RemoteCommand,
  SharedFile,
  SystemHealth,
  User,
} from "../types";

const TOKEN_KEY = "kusuri_jwt_token";

export function getStoredToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function setStoredToken(token: string | null): void {
  if (typeof window === "undefined") return;
  if (token) {
    localStorage.setItem(TOKEN_KEY, token);
  } else {
    localStorage.removeItem(TOKEN_KEY);
  }
}

export class ApiError extends Error {
  status: number;
  data: any;

  constructor(message: string, status: number, data?: any) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.data = data;
  }
}

async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const baseUrl = getApiBaseUrl();
  const url = `${baseUrl}${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

  const headers = new Headers(options.headers || {});
  const token = getStoredToken();

  if (token && !headers.has("Authorization")) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  // Only set Content-Type to JSON if not uploading FormData (which sets multipart boundary automatically)
  if (!(options.body instanceof FormData) && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  let response: Response;
  try {
    response = await fetch(url, {
      ...options,
      headers,
    });
  } catch (err: any) {
    throw new ApiError(
      `Gagal terhubung ke backend (${baseUrl}). Pastikan server backend Kusuri aktif.`,
      0,
      err
    );
  }

  if (!response.ok) {
    let errorMessage = `HTTP Error ${response.status}`;
    let errorData = null;
    try {
      errorData = await response.json();
      if (errorData && typeof errorData.message === "string") {
        errorMessage = errorData.message;
      }
    } catch {
      // response is not JSON
      try {
        const text = await response.text();
        if (text) errorMessage = text;
      } catch {
        // fallback
      }
    }

    if (response.status === 401) {
      // Token might be expired
      setStoredToken(null);
    }

    throw new ApiError(errorMessage, response.status, errorData);
  }

  if (response.status === 204) {
    return {} as T;
  }

  return response.json();
}

export const api = {
  auth: {
    async register(username: string, password: string): Promise<AuthResponse> {
      const data = await request<AuthResponse>("/api/auth/register", {
        method: "POST",
        body: JSON.stringify({ username, password }),
      });
      if (data.token) {
        setStoredToken(data.token);
      }
      return data;
    },

    async login(username: string, password: string): Promise<AuthResponse> {
      const data = await request<AuthResponse>("/api/auth/login", {
        method: "POST",
        body: JSON.stringify({ username, password }),
      });
      if (data.token) {
        setStoredToken(data.token);
      }
      return data;
    },

    async me(): Promise<User> {
      return request<User>("/api/auth/me", { method: "GET" });
    },

    logout(): void {
      setStoredToken(null);
    },
  },

  devices: {
    async list(): Promise<Device[]> {
      return request<Device[]>("/api/devices", { method: "GET" });
    },

    async createPairingCode(
      payload: CreatePairingCodeRequest = {}
    ): Promise<PairingCodeResponse> {
      return request<PairingCodeResponse>("/api/devices/pairing-code", {
        method: "POST",
        body: JSON.stringify(payload),
      });
    },

    async update(id: string, name: string): Promise<void> {
      return request<void>(`/api/devices/${id}`, {
        method: "PATCH",
        body: JSON.stringify({ name }),
      });
    },

    async delete(id: string): Promise<void> {
      return request<void>(`/api/devices/${id}`, {
        method: "DELETE",
      });
    },

    async getSpecs(id: string): Promise<EnrichedDeviceSpecsResponse> {
      return request<EnrichedDeviceSpecsResponse>(`/api/devices/${id}/specs`, {
        method: "GET",
      });
    },

    async syncCatalog(): Promise<{ success: boolean; message: string }> {
      return request<{ success: boolean; message: string }>(
        "/api/devices/sync-catalog",
        {
          method: "POST",
        }
      );
    },
  },

  clipboard: {
    async list(): Promise<ClipboardItem[]> {
      return request<ClipboardItem[]>("/api/clipboard", { method: "GET" });
    },

    async create(
      content: string,
      contentType: string = "text/plain"
    ): Promise<ClipboardItem> {
      return request<ClipboardItem>("/api/clipboard", {
        method: "POST",
        body: JSON.stringify({ content, content_type: contentType }),
      });
    },

    async delete(id: string): Promise<void> {
      return request<void>(`/api/clipboard/${id}`, {
        method: "DELETE",
      });
    },
  },

  files: {
    async list(): Promise<SharedFile[]> {
      return request<SharedFile[]>("/api/files", { method: "GET" });
    },

    async upload(file: File, targetDeviceId?: string): Promise<SharedFile> {
      const formData = new FormData();
      formData.append("file", file);
      if (targetDeviceId) {
        formData.append("target_device_id", targetDeviceId);
      }

      return request<SharedFile>("/api/files/upload", {
        method: "POST",
        body: formData,
      });
    },

    async delete(id: string): Promise<void> {
      return request<void>(`/api/files/${id}`, {
        method: "DELETE",
      });
    },

    getDownloadUrl(id: string): string {
      const baseUrl = getApiBaseUrl();
      const token = getStoredToken();
      const tokenParam = token ? `?token=${encodeURIComponent(token)}` : "";
      return `${baseUrl}/api/files/${id}/download${tokenParam}`;
    },
  },

  commands: {
    async list(): Promise<RemoteCommand[]> {
      return request<RemoteCommand[]>("/api/commands", { method: "GET" });
    },

    async send(
      targetDeviceId: string,
      command: string,
      payload?: any
    ): Promise<RemoteCommand> {
      return request<RemoteCommand>("/api/commands", {
        method: "POST",
        body: JSON.stringify({
          target_device_id: targetDeviceId,
          command,
          payload: payload ?? {},
        }),
      });
    },
  },

  system: {
    async health(): Promise<SystemHealth> {
      return request<SystemHealth>("/api/health", { method: "GET" });
    },
  },

  notifications: {
    async list(params?: {
      device_id?: string;
      search?: string;
      limit?: number;
    }): Promise<DeviceNotification[]> {
      const query = new URLSearchParams();
      if (params?.device_id) query.append("device_id", params.device_id);
      if (params?.search) query.append("search", params.search);
      if (params?.limit) query.append("limit", params.limit.toString());

      const qs = query.toString() ? `?${query.toString()}` : "";
      return request<DeviceNotification[]>(`/api/notifications${qs}`, {
        method: "GET",
      });
    },
  },
};
