CREATE TABLE project(
    id serial PRIMARY KEY,
    metadata jsonb,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    description jsonb,
    visible boolean,
    adult boolean
);

CREATE TABLE project_content(
    id serial PRIMARY KEY,
    content jsonb,
    project_id INT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

CREATE TABLE project_content_thumbnail(
    id serial PRIMARY KEY,
    content jsonb,
    project_id INT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

ALTER TABLE project_content_thumbnail ADD CONSTRAINT unique_project_id UNIQUE (project_id);
