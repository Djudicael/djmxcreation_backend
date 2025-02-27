CREATE TABLE about(
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        first_name VARCHAR (50) UNIQUE NOT NULL,
        last_name VARCHAR (50) UNIQUE NOT NULL,
        photo jsonb,
        description jsonb
    );


