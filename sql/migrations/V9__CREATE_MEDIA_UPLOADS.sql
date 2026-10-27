CREATE TABLE media_upload (
    id UUID PRIMARY KEY,
    project_id UUID REFERENCES project(id) ON DELETE SET NULL,
    object_key TEXT NOT NULL UNIQUE,
    upload_id TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    file_size BIGINT NOT NULL CHECK (file_size > 0),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'complete', 'cancelled')),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);
CREATE INDEX media_upload_pending ON media_upload(expires_at) WHERE status = 'pending';
