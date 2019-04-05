#!/usr/bin/env bash
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo build --verbose
cargo test --verbose
cargo clippy --verbose -- -D warnings

# Code format is optional until we can make it work.
cargo fmt --verbose -- --check || true
# But for (new) crates that pass it we do not want to avoid regressions.
#cargo fmt --verbose --package replicante_util_failure -- --check
