-- Migration 0004: Support rich desktop collective telemetry & specifications
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS cpu_model VARCHAR(200);
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS cpu_cores INT;
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS cpu_logical_cores INT;
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS cpu_frequency_mhz BIGINT;
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS gpu_model VARCHAR(200);
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS hostname VARCHAR(150);
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS architecture VARCHAR(50);
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS uptime_formatted VARCHAR(100);
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS disks_info JSONB DEFAULT '[]'::jsonb;
ALTER TABLE device_hardware_telemetry ADD COLUMN IF NOT EXISTS network_interfaces JSONB DEFAULT '[]'::jsonb;
