#!/usr/bin/env bash

set -euxo pipefail

# WTX

cargo check --all-features --all-targets

RUSTDOCFLAGS="-Dwarnings" cargo doc --all-features
