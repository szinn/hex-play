CREATE TABLE user_info (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  user_token UUID NOT NULL REFERENCES users(token) ON DELETE CASCADE,
  age SMALLINT NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX unq_user_token ON user_info (user_token);

SELECT trigger_updated_at('user_info');
