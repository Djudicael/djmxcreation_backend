CREATE TABLE project(
    id serial PRIMARY KEY,
    metadata jsonb,
    created_on TIMESTAMP NOT NULL,
    updated_on TIMESTAMP,
    description jsonb,
    visible boolean
);

CREATE TABLE project_content(
    id serial PRIMARY KEY,
    content jsonb,
    project_id INT NOT NULL,
    created_on TIMESTAMP NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE
);