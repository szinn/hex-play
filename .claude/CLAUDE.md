# Rust Project to Experiment With Hexagonal Architecture

## One-time Setup

This project uses the nightly toolchain for formatting and a few extra tools including
[mise](https://mise.jdx.dev) and [just](https://just.systems). To install/update the tools:

```bash
just install-tools
```

## Commands

Install tools: `just install-tools`
Format code: `just fmt`
Run clippy: `just clippy`
Run tests: `just test`
Run insta Tests: `just insta`
Clean workspace: `just clean`
Create changelog: `just changelog`
Create database: `just create-database`
Extract database entities: `just entities`

## Directory Structure

The project uses Rust workspaces with all crates in `crates/`.

```
hex-play/
├── .cargo/ # Cargo configuration
├── .claude/ # Claude Code settings
├── .config/ # Project config
├── crates/ # Workspace crates
│ └── cli/ # hex-play CLI application
├── Cargo.toml # Workspace manifest
├── Cargo.lock
├── .justfile # Just command runner tasks
├── deny.toml # cargo-deny configuration
└── rustfmt.toml # Rust formatting config
```

## Workflows

**Before committing:**

- run tests to make sure everything passes
- run clippy for linting
- format code

## Conventions

- **Commits:**
  - Follow conventional commits (`chore(app): description`)
  - use `jujutsu`, not `git`
- **Secrets:**
  - Secrets should be encrypted with `sops`, never commit secrets
