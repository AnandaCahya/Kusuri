// TypeScript types matching Kusuri Backend models & REQ.md specifications

export type DeviceType = "android" | "windows" | "linux" | "web" | "other" | string;

export interface User {
  id: string;
  username: string;
  created_at: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Device {
  id: string;
  user_id: string;
  name: string;
  device_type: DeviceType;
  is_online: boolean;
  last_seen_at?: string | null;
  created_at: string;
  raw_model?: string | null;
  marketing_name?: string | null;
  retail_branding?: string | null;
  chipset?: string | null;
}

export interface PairingCodeResponse {
  pairing_id: string;
  code: string;
  expires_at: string;
  expires_in_seconds: number;
  max_attempts: number;
}

export interface CreatePairingCodeRequest {
  device_name?: string;
  device_type?: string;
}

export interface GsmArenaSpecs {
  id?: string;
  device_name?: string;
  marketing_name?: string;
  brand?: string;
  retail_branding?: string;
  chipset?: string | null;
  cpu?: string | null;
  gpu?: string | null;
  display_type?: string | null;
  display_size?: string | null;
  display_resolution?: string | null;
  main_camera?: string | null;
  camera_spec?: string | null;
  selfie_camera?: string | null;
  battery_type?: string | null;
  battery_spec?: string | null;
  battery_charging?: string | null;
  charging_spec?: string | null;
  dimensions?: string | null;
  weight?: string | null;
  memory_spec?: string | null;
  os_original?: string | null;
  picture_url?: string | null;
  image_url?: string | null;
  gsmarena_url?: string | null;
  raw_specs?: {
    sources?: string[];
    source_urls?: Record<string, string>;
    details?: Record<string, string>;
  } | null;
}

export interface LiveTelemetry {
  ram_total_mb?: number | null;
  ram_used_mb?: number | null;
  storage_total_mb?: number | null;
  storage_used_mb?: number | null;
  battery_percent?: number | null;
  battery_charging?: boolean | null;
  battery_temp_c?: number | null;
  os_version?: string | null;
  android_sdk?: number | null;
  imei1?: string | null;
  imei2?: string | null;
  updated_at?: string | null;
}

export interface EnrichedDeviceSpecsResponse {
  device_id: string;
  device_name?: string;
  name?: string;
  device_type: string;
  raw_model?: string | null;
  marketing_name?: string | null;
  retail_branding?: string | null;
  chipset?: string | null;
  is_online?: boolean;
  specs?: GsmArenaSpecs | null;
  catalog_specs?: GsmArenaSpecs | null;
  telemetry?: LiveTelemetry | null;
  live_hardware?: LiveTelemetry | null;
}

export interface ClipboardItem {
  id: string;
  user_id: string;
  source_device_id?: string | null;
  source_device_name?: string | null;
  content: string;
  content_type: string;
  created_at: string;
}

export interface SharedFile {
  id: string;
  user_id: string;
  source_device_id?: string | null;
  source_device_name?: string | null;
  target_device_id?: string | null;
  original_filename: string;
  file_size: number;
  mime_type: string;
  download_url: string;
  created_at: string;
}

export type CommandStatus = "pending" | "sent" | "completed" | "failed" | string;

export interface RemoteCommand {
  id: string;
  user_id: string;
  source_device_id?: string | null;
  source_device_name?: string | null;
  target_device_id: string;
  target_device_name?: string | null;
  command: string;
  payload?: any;
  status: CommandStatus;
  response?: any;
  created_at: string;
  updated_at: string;
}

export interface SystemHealth {
  status: string;
  service: string;
  version: string;
  timestamp?: string;
}

export type WsConnectionStatus = "connecting" | "connected" | "disconnected" | "error";

export type WsServerEvent =
  | { type: "pong" }
  | { type: "device_connected"; payload: { device_id: string; name: string; device_type: string } }
  | { type: "device_disconnected"; payload: { device_id: string } }
  | {
      type: "clipboard_updated";
      payload: {
        id: string;
        content: string;
        content_type: string;
        source_device_id?: string | null;
        source_device_name?: string | null;
        created_at: string;
      };
    }
  | { type: "device_status_updated"; payload: { device_id: string; data: any } }
  | { type: "device_specs_updated"; payload: { device_id: string; specs: EnrichedDeviceSpecsResponse } }
  | {
      type: "file_available";
      payload: {
        file_id: string;
        original_filename: string;
        file_size: number;
        mime_type: string;
        download_url: string;
        source_device_id?: string | null;
      };
    }
  | {
      type: "command_result_received";
      payload: {
        command_id: string;
        status: string;
        response: any;
        device_id: string;
      };
    }
  | { type: "relay_event"; payload: { source_device_id?: string; event: string; data: any } }
  | { type: "error"; payload: { message: string } };
