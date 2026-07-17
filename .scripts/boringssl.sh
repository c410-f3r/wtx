#!/usr/bin/env bash

set -euxo pipefail

# Successful tests expect no output but this is is kept for debugging.
#export RUST_LOG=trace

if [ ! -d "./boringssl" ]; then
    git clone --depth 1 --branch 0.20260713.0 https://github.com/google/boringssl
    rm boringssl/.git
fi

if [ ! -f "./boringssl-config.json" ]; then
    cargo run --bin boringssl-config --features boringssl-config
fi

cargo build --bin boringssl --features boringssl
cd boringssl/ssl/test/runner
go test -c
./runner.test \
    -num-workers 1 \
    -pipe \
    -shim-config ../../../../boringssl-config.json \
    -shim-path ../../../../target/debug/boringssl \
    -test.v