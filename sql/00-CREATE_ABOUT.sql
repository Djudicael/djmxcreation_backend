CREATE TABLE about(
    id serial PRIMARY KEY,
    first_name VARCHAR (50) UNIQUE NOT NULL,
    last_name VARCHAR (50) UNIQUE NOT NULL,
    photo jsonb,
    description jsonb
);