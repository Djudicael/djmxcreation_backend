CREATE TABLE project_spotlight(
    id serial PRIMARY KEY,
    project_id INT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);

ALTER TABLE project_spotlight ADD CONSTRAINT unique_project_id UNIQUE (project_id);