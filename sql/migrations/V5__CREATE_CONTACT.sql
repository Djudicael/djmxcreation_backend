CREATE TABLE IF NOT EXISTS contact(
    id UUID PRIMARY KEY,  -- Removed DEFAULT gen_random_uuid()
    description jsonb
);


INSERT INTO contact (id, description) 
SELECT '550e8400-e29b-41d4-a716-446655440000', NULL 
WHERE NOT EXISTS (SELECT 1 FROM contact);
