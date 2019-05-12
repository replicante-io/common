#!/usr/bin/env bash
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --verbose -- --check
