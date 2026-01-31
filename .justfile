#!/usr/bin/env -S just --justfile

set unstable := true
set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

[private]
default:
    just -l

[doc('Install tooling for contributing to this project')]
install-tools:
  mise install
  rustup toolchain add nightly
  cargo install --locked cargo-insta cargo-nextest

[doc('Format code and documentation')]
fmt:
  cargo +nightly fmt --all
  prettier --config .config/prettierrc --ignore-path .gitignore --ignore-path .config/prettierignore --log-level warn -w .

[doc('Update CHANGELOG.md')]
changelog:
  RUST_LOG= git-cliff --config .config/cliff.toml > CHANGELOG.md
  just fmt

[doc('Build all applications')]
build:
  cargo build --bin hex-play --bin migrator

[doc('Run Clippy on codebase for linting')]
clippy:
  cargo +nightly clippy --workspace --all-targets

[doc('Update rust crate dependencies')]
deps:
  cargo upgrade

[doc('Run all tests using nextest')]
test:
  cargo nextest run --workspace

[doc('Run all tests using insta')]
insta:
  cargo insta test --workspace --test-runner nextest

[doc('Clean project workspace')]
clean:
  cargo clean

[doc('Database Admin')]
database:
  PGUSER=$PGADMINUSER PGPASSWORD=$PGADMINPASSWORD PGDATABASE= psql-18

[doc('Create the database')]
create-database:
  #!/usr/bin/env bash
  set -euo pipefail

  SQL="""
    DROP DATABASE IF EXISTS "$PGDATABASE";
    DROP ROLE IF EXISTS "$PGUSER";

    CREATE ROLE "$PGUSER" WITH
      LOGIN
      NOSUPERUSER
      INHERIT
      NOCREATEDB
      NOCREATEROLE
      NOREPLICATION
      PASSWORD '$PGPASSWORD';

    CREATE DATABASE "$PGDATABASE"
      WITH
      OWNER = "$PGUSER"
      ENCODING = 'UTF8'
      LC_COLLATE = 'C'
      LC_CTYPE = 'C'
      TABLESPACE = pg_default
      CONNECTION LIMIT = -1
      IS_TEMPLATE = False;

    GRANT TEMPORARY, CONNECT ON DATABASE "$PGDATABASE" TO PUBLIC;

    GRANT ALL ON DATABASE "$PGDATABASE" TO "$PGUSER";
  """
  echo $SQL | PGUSER=$PGADMINUSER PGPASSWORD=$PGADMINPASSWORD PGDATABASE= psql-18 postgres

[doc('Redo all migrations')]
migrations:
  DATABASE_URL=$HPLAY__DATABASE__DATABASE_URL cargo run --bin migrator -- down
  DATABASE_URL=$HPLAY__DATABASE__DATABASE_URL cargo run --bin migrator -- up

[doc('Create entity classes')]
entities:
  DATABASE_URL=$HPLAY__DATABASE__DATABASE_URL cargo run --bin migrator -- up
  DATABASE_URL=$HPLAY__DATABASE__DATABASE_URL sea-orm-cli generate entity -o crates/database/src/entities --with-serde both --entity-format dense
