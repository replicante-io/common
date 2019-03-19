#!/usr/bin/env bash
set -ex

ln -f -s Cargo.workspace.toml Cargo.toml
cargo build --verbose
cargo test --verbose
