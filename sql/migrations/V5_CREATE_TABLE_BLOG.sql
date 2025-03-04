CREATE TABLE IF NOT EXISTS blog(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metadata jsonb,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    description jsonb,
    visible boolean,
    adult boolean
);

CREATE TABLE IF NOT EXISTS blog_content(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content jsonb,
    blog_id UUID NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_blog FOREIGN KEY (blog_id) REFERENCES blog (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS blog_content_thumbnail(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content jsonb,
    blog_id UUID NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_blog FOREIGN KEY (blog_id) REFERENCES blog (id) ON DELETE CASCADE
);

ALTER TABLE blog_content_thumbnail ADD CONSTRAINT unique_blog_id UNIQUE (blog_id);
