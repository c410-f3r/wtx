#!/usr/bin/env bash

set -euxo pipefail

export rt='cargo run --bin rust-tools -- --template you-rust'

export CARGO_TARGET_DIR="$($rt target-dir)"
export RUST_BACKTRACE=1
export RUSTFLAGS="$($rt rust-flags)"
