CREATE TABLE contact(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description jsonb
);


INSERT INTO contact (description) SELECT NULL WHERE NOT EXISTS (SELECT 1 FROM contact);