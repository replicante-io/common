#!/usr/bin/env bash
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo build --verbose
cargo test --verbose
cargo clippy --verbose -- -D warnings
# Code format is optional until we can make it work.
cargo fmt --verbose -- --check || true
