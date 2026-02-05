# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### <!-- 01 -->Features

- _(api)_ Api framework
- _(api)_ Basic GRPC server
- _(api)_ Add gRPC error mapping from core ErrorKind to tonic Status
- _(api)_ Add gRPC UserService and refactor to SystemService
- _(api)_ Add version, created_at, updated_at to User responses
- _(api,core)_ Adding an api route to core services
- _(api,core)_ User CRUD api
- _(api,core,database)_ Implement user CRUD use cases
- _(api,core,database)_ Add UUID token field to users
- _(api,core,database)_ Add age field to User with user_info storage
- _(cli)_ Config framework
- _(cli)_ Command arguments framework
- _(cli,core,database)_ Switch to Entity First Workflow
- _(core)_ Add transaction helper functions with auto commit/rollback
- _(core,database)_ Beginning of repository and transactions
- _(core,database)_ Beginning of User service
- _(core,database)_ Added update for User service
- _(core,database)_ CoreServices framework
- _(database)_ Database framework
- Exploring entity-first vs migrations
- Filling in basic CLI for user CRUD operations

### <!-- 06 -->Miscellaneous Tasks

- _(api)_ Add HTTP endpoint tests for user routes
- _(api)_ Simplify HTTP server setup with axum::serve
- _(api)_ Add comprehensive tests for gRPC services
- _(api,core)_ Extract shared MockUserService to core test_support
- _(api,core)_ Move API infrastructure errors to api crate
- _(api,core,database)_ Improve code quality and reduce duplication
- _(api,core,database)_ Improve error handling, add newtypes, and consolidate test infrastructure
- _(api,core,database)_ Adopt Email and Age newtypes in domain models
- _(config)_ Migrate config .renovaterc.json5 ([#3](https://github.com/szinn/rust-arch/issues/3))
- _(core)_ Improve module organization and add NewUser model
- _(core)_ Extend transaction macros to support multiple services
- _(core)_ Restructure modules - extract repositories from services
- _(core,api)_ Move UserService to services module
- _(core,database)_ Improve delete_user with optimistic locking
- _(core,database)_ Add unit tests for user use cases and adapter
- _(core,database)_ Rename UserService to UserRepository
- _(core,database)_ Use derive_builder for RepositoryService construction
- _(core,database)_ Remove user_info table and consolidate age into users
- _(core,database,api)_ Use idiomatic combinators and flatten module structure
- _(database)_ Replace mock database tests with SQLite in-memory
- _(deps)_ Update to sea-orm 2.0.0-rc
- _(deps)_ Update renovatebot/github-action action to v40.3.6
- _(deps)_ Update rust crate clap to 4.5.56 ([#4](https://github.com/szinn/rust-arch/issues/4))
- _(deps)_ Update renovatebot/github-action action to v46.0.1 ([#9](https://github.com/szinn/rust-arch/issues/9))
- _(deps)_ Update Rust dependencies
- _(deps)_ Change Renovate cargo rangeStrategy from replace to bump
- _(deps)_ Update crate anyhow to 1.0.101 ([#11](https://github.com/szinn/rust-arch/issues/11))
- _(renovate)_ Add renovate support
- _(renovate)_ Add renovate support
- Formatting
- Configure Renovate to update Cargo.toml version constraints
- Configure Renovate to update Cargo.toml version constraints
- Upgrade crate dependencies
- Use find_with_related for list_users
- Update crates
