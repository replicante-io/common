#!/usr/bin/env sh
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo clippy --verbose -- -D warnings
