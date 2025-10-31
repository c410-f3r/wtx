#!/usr/bin/env bash

set -euxo pipefail

# WTX

cargo check --all-features --all-targets

# WTX Docs

rustup default nightly-2025-10-31
cargo clean --target-dir mdbook-target
cargo build --all-features --target-dir mdbook-target
mdbook test -L mdbook-target/debug/deps wtx-docs

RUSTDOCFLAGS="-Dwarnings" cargo doc --all-features
