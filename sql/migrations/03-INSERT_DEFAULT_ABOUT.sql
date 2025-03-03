INSERT INTO about (id, first_name, last_name) 
SELECT '550e8400-e29b-41d4-a716-446655440000', 'Your', 'Name' 
WHERE NOT EXISTS (SELECT 1 FROM about);