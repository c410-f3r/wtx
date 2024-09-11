#!/usr/bin/env bash

# WTX

cargo check --all-features --all-targets

# WTX Docs

rustup default nightly-2024-09-07
cargo clean --target-dir mdbook-target
cargo build --all-features --target-dir mdbook-target
mdbook test -L mdbook-target/debug/deps wtx-docs