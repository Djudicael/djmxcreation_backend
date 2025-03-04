CREATE TABLE IF NOT EXISTS project_spotlight(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

ALTER TABLE project_spotlight ADD CONSTRAINT unique_project_id UNIQUE (project_id);