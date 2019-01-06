CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE
);

SELECT diesel_manage_updated_at('users');
