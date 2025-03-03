CREATE TABLE IF NOT EXISTS about (
    id UUID PRIMARY KEY,  -- Removed DEFAULT gen_random_uuid()
    first_name VARCHAR(50) UNIQUE NOT NULL,
    last_name VARCHAR(50) UNIQUE NOT NULL,
    photo JSONB,
    description JSONB
);