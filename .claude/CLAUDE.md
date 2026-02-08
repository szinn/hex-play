# Rust Project to Experiment With Hexagonal Architecture

## One-time Setup

This project uses edition 2024 with `rust-version = "1.85"` and the nightly toolchain for
formatting and clippy. Extra tools include [mise](https://mise.jdx.dev) and
[just](https://just.systems). To install/update the tools:

```bash
just install-tools
```

## Commands

- Install tools: `just install-tools`
- Build: `just build`
- Format code: `just fmt`
- Update rust crate dependencies: `just deps`
- Run clippy: `just clippy`
- Run tests: `just test`
- Run insta tests: `just insta`
- Clean workspace: `just clean`
- Create changelog: `just changelog`
- Database admin: `just database`
- Create database: `just create-database`
- Redo all migrations: `just migrations`
- Extract database entities: `just entities`

## Architecture

This project follows hexagonal (ports & adapters) architecture. Dependencies point inward
toward the core domain. Never introduce dependencies from `core` to outer crates.

```
crates/
├── core/      # Domain layer: business logic, domain models, and port traits (interfaces)
├── database/  # Adapter: implements persistence ports defined in core (SeaORM/Postgres)
├── api/       # Adapter: HTTP interface via Axum, calls into core ports
└── cli/       # Application entry point, wires adapters to ports
```

Only `crates/cli` is a direct workspace member. The other crates are pulled in transitively
as path dependencies.

## Frontend

The frontend is built using Dioxus. See @.claude/Dioxus.md for more info.

## Database

The project uses PostgreSQL with SeaORM. A running Postgres instance is required for
database-related commands. The following environment variables must be set:

- `PGUSER`, `PGPASSWORD`, `PGDATABASE` — used by `just create-database` and `just database`
- `PGADMINUSER`, `PGADMINPASSWORD` — admin credentials for database creation
- `HPLAY__DATABASE__DATABASE_URL` — SeaORM connection string for migrations and entity generation

Secrets should be encrypted with `sops` and never committed.

## Workflows

**Before committing:**

- Run tests to make sure everything passes: `just test`
- Run clippy for linting: `just clippy`
- Format code: `just fmt`

## Testing

- Use `cargo-nextest` as the test runner (`just test`)
- Use `cargo-insta` for snapshot testing (`just insta`) when asserting against larger or
  structured output; use regular assertions for simple value checks
- Tests live alongside source code in `#[cfg(test)]` modules

## Conventions

- **Commits:**
  - Follow conventional commits with crate-based scopes sorted: `type(scope): description`
  - Valid scopes: `api`, `cli`, `core`, `database` (match crate names)
  - Use `jj` (jujutsu) for version control, not `git`
  - Key commands: `jj commit`, `jj describe`, `jj new`, `jj log`, `jj status`
- **Error handling:**
  - Use `thiserror` for typed errors in library crates (`core`, `api`, `database`)
  - Use `anyhow` for ad-hoc errors in the binary crate (`cli`)
- **Secrets:**
  - Secrets should be encrypted with `sops`, never commit secrets
- **Dependencies:**
  - All crate dependencies must be defined in the root `Cargo.toml` under `[workspace.dependencies]`
  - Individual crates reference them with `crate-name.workspace = true`
  - In root `Cargo.toml`: version-only deps use inline format (`anyhow = "1.0.100"`), but deps
    with features or other options use section format:

        ```toml
        [workspace.dependencies.uuid]
        version = "1"
        features = ["v4", "serde"]
        ```
