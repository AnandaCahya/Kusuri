-- Google Supported Devices catalog table
CREATE TABLE IF NOT EXISTS google_supported_devices (
    id BIGSERIAL PRIMARY KEY,
    retail_branding VARCHAR(100),
    marketing_name VARCHAR(200) NOT NULL,
    device VARCHAR(100) NOT NULL, -- e.g. 'Infinix-X6881'
    model VARCHAR(100)            -- e.g. 'Infinix X6881'
);

CREATE INDEX IF NOT EXISTS idx_gsd_device ON google_supported_devices(LOWER(device));
CREATE INDEX IF NOT EXISTS idx_gsd_model ON google_supported_devices(LOWER(model));
CREATE INDEX IF NOT EXISTS idx_gsd_branding ON google_supported_devices(LOWER(retail_branding));

-- GSM Arena / Device Catalog Specifications table
CREATE TABLE IF NOT EXISTS device_catalog_specs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    marketing_name VARCHAR(200) UNIQUE NOT NULL,
    retail_branding VARCHAR(100),
    gsmarena_url VARCHAR(350),
    chipset VARCHAR(200),
    cpu VARCHAR(250),
    gpu VARCHAR(200),
    display_type VARCHAR(150),
    display_size VARCHAR(100),
    display_resolution VARCHAR(100),
    os_original VARCHAR(150),
    battery_spec VARCHAR(150),
    charging_spec VARCHAR(150),
    camera_spec VARCHAR(250),
    memory_spec VARCHAR(200),
    image_url VARCHAR(350),
    raw_specs JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Alter devices table to support marketing names and specifications
ALTER TABLE devices ADD COLUMN IF NOT EXISTS raw_model VARCHAR(100);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS marketing_name VARCHAR(200);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS retail_branding VARCHAR(100);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS chipset VARCHAR(200);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS specs_id UUID REFERENCES device_catalog_specs(id) ON DELETE SET NULL;

-- Live hardware telemetry details from client (RAM, Storage, Battery, IMEI, etc.)
CREATE TABLE IF NOT EXISTS device_hardware_telemetry (
    device_id UUID PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
    imei1 VARCHAR(50),
    imei2 VARCHAR(50),
    serial_number VARCHAR(100),
    ram_total_mb BIGINT,
    ram_used_mb BIGINT,
    ram_free_mb BIGINT,
    storage_total_mb BIGINT,
    storage_used_mb BIGINT,
    storage_free_mb BIGINT,
    battery_percent REAL,
    battery_charging BOOLEAN,
    battery_temp_c REAL,
    battery_health VARCHAR(50),
    os_version VARCHAR(100),
    android_sdk INT,
    kernel_version VARCHAR(150),
    screen_resolution VARCHAR(50),
    screen_refresh_rate REAL,
    ip_address VARCHAR(50),
    network_type VARCHAR(50),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
