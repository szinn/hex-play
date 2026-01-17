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
  git-cliff --config .config/cliff.toml > CHANGELOG.md
  just fmt

[doc('Run Clippy on codebase for linting')]
clippy:
  cargo +nightly clippy --workspace --all-targets

[doc('Run all tests using nextest')]
test:
  cargo nextest run --workspace

[doc('Run all tests using insta')]
insta:
  cargo insta test --workspace --test-runner nextest

[doc('Clean project workspace')]
clean:
  cargo clean
