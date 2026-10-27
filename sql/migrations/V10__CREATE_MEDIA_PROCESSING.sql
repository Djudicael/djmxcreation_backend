-- Originals remain canonical. Jobs/assets are optional accelerators, not media records.
CREATE TABLE media_optimization (
    media_id UUID PRIMARY KEY REFERENCES project_content(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','processing','ready','failed','skipped')),
    step INT NOT NULL DEFAULT 0 CHECK(step BETWEEN 0 AND 4),
    attempts INT NOT NULL DEFAULT 0,
    lease_token UUID,
    lease_until TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT,
    original_width INT,
    original_height INT
);
CREATE TABLE media_asset (
    media_id UUID NOT NULL,
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('original','variant')),
    target_width INT,
    width INT,
    height INT,
    mime_type TEXT,
    ready BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY(bucket,object_key)
);
CREATE INDEX media_asset_owner ON media_asset(media_id);
CREATE TABLE media_cleanup (
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 0,
    lease_token UUID,
    lease_until TIMESTAMPTZ,
    PRIMARY KEY(bucket,object_key)
);
CREATE INDEX media_cleanup_due ON media_cleanup(available_at);

CREATE FUNCTION register_media_original() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.content->>'bucket_name' IS NOT NULL AND NEW.content->>'file_name' IS NOT NULL THEN
        INSERT INTO media_asset(media_id,bucket,object_key,kind,ready)
        VALUES(NEW.id,NEW.content->>'bucket_name',NEW.content->>'file_name','original',TRUE)
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER register_media_original AFTER INSERT ON project_content FOR EACH ROW EXECUTE FUNCTION register_media_original();
INSERT INTO media_asset(media_id,bucket,object_key,kind,ready)
SELECT id,content->>'bucket_name',content->>'file_name','original',TRUE FROM project_content
WHERE content->>'bucket_name' IS NOT NULL AND content->>'file_name' IS NOT NULL ON CONFLICT DO NOTHING;

CREATE FUNCTION schedule_media_cleanup() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE variant_due TIMESTAMPTZ;
BEGIN
    -- A processing step holds the content row while writing. Keep an additional
    -- grace period for storage requests interrupted by a terminated invocation.
    SELECT GREATEST(NOW(),COALESCE(lease_until,NOW())) + INTERVAL '30 seconds'
        INTO variant_due FROM media_optimization WHERE media_id=OLD.id;
    INSERT INTO media_cleanup(bucket,object_key,available_at)
    SELECT bucket,object_key,CASE WHEN kind='original' THEN NOW() ELSE COALESCE(variant_due,NOW()+INTERVAL '30 seconds') END
    FROM media_asset WHERE media_id=OLD.id
    ON CONFLICT(bucket,object_key) DO UPDATE SET available_at=GREATEST(media_cleanup.available_at,EXCLUDED.available_at);
    DELETE FROM project_content_thumbnail WHERE project_id=OLD.project_id AND content->>'file_name'=OLD.content->>'file_name';
    RETURN OLD;
END $$;
CREATE TRIGGER schedule_media_cleanup BEFORE DELETE ON project_content FOR EACH ROW EXECUTE FUNCTION schedule_media_cleanup();

CREATE FUNCTION enqueue_media_optimization(target UUID) RETURNS VOID LANGUAGE SQL AS $$
    INSERT INTO media_optimization(media_id)
    SELECT id FROM project_content WHERE id=target AND
      (content->>'mime_type' LIKE 'image/%' OR (content->>'mime_type' IS NULL AND content->>'file_name' ~* '[.](png|jpe?g|webp|avif|gif)$'))
    ON CONFLICT DO NOTHING;
$$;
