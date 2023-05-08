CREATE TABLE blog(
    id serial PRIMARY KEY,
    metadata jsonb,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    description jsonb,
    visible boolean,
    adult boolean
);

CREATE TABLE blog_content(
    id serial PRIMARY KEY,
    content jsonb,
    blog_id INT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_blog FOREIGN KEY (blog_id) REFERENCES blog (id) ON DELETE CASCADE
);

CREATE TABLE blog_content_thumbnail(
    id serial PRIMARY KEY,
    content jsonb,
    blog_id INT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_blog FOREIGN KEY (blog_id) REFERENCES blog (id) ON DELETE CASCADE
);

ALTER TABLE blog_content_thumbnail ADD CONSTRAINT unique_blog_id UNIQUE (blog_id);
