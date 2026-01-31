CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  token UUID         NOT NULL DEFAULT gen_random_uuid(),
  name  VARCHAR(128) NOT NULL,
  email VARCHAR(128) NOT NULL,

  version BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX unq_email ON users (email);
CREATE UNIQUE INDEX unq_token ON users (token);

SELECT trigger_updated_at_and_version('users');

CREATE OR REPLACE FUNCTION prevent_token_update()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.token IS DISTINCT FROM NEW.token THEN
        RAISE EXCEPTION 'token field cannot be updated';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_token_update_trigger
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION prevent_token_update();
