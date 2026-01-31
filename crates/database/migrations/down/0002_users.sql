DROP INDEX IF EXISTS unq_email;
DROP INDEX IF EXISTS unq_token;
DROP TRIGGER IF EXISTS prevent_token_update_trigger ON users;
DROP FUNCTION IF EXISTS prevent_token_update();
DROP TRIGGER set_updated_at_and_version ON users;
DROP TABLE users;
