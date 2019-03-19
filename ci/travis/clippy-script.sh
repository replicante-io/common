#!/usr/bin/env bash
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo clippy --verbose -- -D warnings
