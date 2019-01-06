CREATE TABLE channels (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users,
  title TEXT NOT NULL,
  is_public BOOL NOT NULL
);

SELECT diesel_manage_updated_at('channels');
