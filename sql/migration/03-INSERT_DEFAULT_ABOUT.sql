INSERT INTO about (first_name, last_name) SELECT 'Your', 'Name' WHERE NOT EXISTS (SELECT 1 FROM about);