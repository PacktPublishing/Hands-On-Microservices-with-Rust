CREATE TABLE memberships (
  id SERIAL PRIMARY KEY,
  channel_id INTEGER NOT NULL REFERENCES channels,
  user_id INTEGER NOT NULL REFERENCES users
);

SELECT diesel_manage_updated_at('memberships');
