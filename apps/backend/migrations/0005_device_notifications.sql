-- Migration 0005: Cross-platform device notification history
CREATE TABLE IF NOT EXISTS device_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES devices(id) ON DELETE SET NULL,
    package_or_app_name VARCHAR(255) NOT NULL,
    app_title VARCHAR(255),
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    subtext VARCHAR(500),
    posted_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_device_notifications_user_posted ON device_notifications(user_id, posted_at DESC);
CREATE INDEX IF NOT EXISTS idx_device_notifications_device_posted ON device_notifications(device_id, posted_at DESC);
