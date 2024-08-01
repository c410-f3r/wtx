#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX Docs

rustup default nightly-2024-07-10
cargo clean --target-dir mdbook-target
cargo build --all-features --target-dir mdbook-target
mdbook test -L mdbook-target/debug/deps wtx-docs