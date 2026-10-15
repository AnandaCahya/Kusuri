-- Add token_hash to devices table
ALTER TABLE devices ADD COLUMN IF NOT EXISTS token_hash VARCHAR(64);

-- Populate token_hash with sha256 of existing device_token if any exist
UPDATE devices 
SET token_hash = encode(sha256(device_token::bytea), 'hex') 
WHERE token_hash IS NULL AND device_token IS NOT NULL;

-- Make token_hash UNIQUE and NOT NULL
ALTER TABLE devices ALTER COLUMN token_hash SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_token_hash ON devices(token_hash);

-- Enhance device_pairing_codes table with attempt counters and UUID session id
ALTER TABLE device_pairing_codes ADD COLUMN IF NOT EXISTS id UUID DEFAULT gen_random_uuid();
ALTER TABLE device_pairing_codes ADD COLUMN IF NOT EXISTS attempts INT NOT NULL DEFAULT 0;
ALTER TABLE device_pairing_codes ADD COLUMN IF NOT EXISTS max_attempts INT NOT NULL DEFAULT 3;

CREATE INDEX IF NOT EXISTS idx_pairing_codes_lookup ON device_pairing_codes(code, expires_at);
