CREATE TABLE IF NOT EXISTS project(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metadata jsonb,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    description jsonb,
    visible boolean,
    adult boolean
);

CREATE TABLE IF NOT EXISTS project_content(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content jsonb,
    project_id UUID NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_content_thumbnail(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content jsonb,
    project_id UUID NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

ALTER TABLE project_content_thumbnail ADD CONSTRAINT unique_content_thumbnail_project_id UNIQUE (project_id);

CREATE INDEX idx_project_visible_adult ON project(visible, adult);
CREATE INDEX idx_project_content_project_id ON project_content(project_id);
CREATE INDEX idx_project_content_thumbnail_project_id ON project_content_thumbnail(project_id);
