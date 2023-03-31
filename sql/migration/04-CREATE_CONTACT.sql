CREATE TABLE contact(
    id serial PRIMARY KEY,
    description jsonb
);


INSERT INTO contact (description) SELECT NULL WHERE NOT EXISTS (SELECT 1 FROM contact);