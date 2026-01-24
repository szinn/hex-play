CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(128) NOT NULL,
  email VARCHAR(128) NOT NULL,

  version BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX unq_email ON users (email);

SELECT trigger_updated_at_and_version('users');
