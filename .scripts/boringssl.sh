#!/usr/bin/env bash

set -euxo pipefail

if [ ! -d "./boringssl" ]; then
    git clone --depth 1 --branch 0.20241209.0 https://github.com/google/boringssl
fi

cargo run --bin boringssl-config --features boringssl-config
cargo build --bin boringssl --features boringssl
cd boringssl/ssl/test/runner
go test -c
./runner.test -shim-config ../../../../boringssl-config.json  -shim-path ../../../../target/debug/boringssl
