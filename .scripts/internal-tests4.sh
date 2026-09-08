#!/usr/bin/env bash

set -euxo pipefail

# WTX

cargo check --all-features --all-targets

# WTX Docs

rustup default nightly-2026-09-03
cargo clean --target-dir mdbook-target
__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1 cargo build --all-features --target-dir mdbook-target
mdbook test -L mdbook-target/debug/deps wtx-docs

RUSTDOCFLAGS="-Dwarnings" cargo doc --all-features
