#!/usr/bin/env bash

set -euxo pipefail

# Successful tests expect no output but this variable is kept for debugging.
#export RUST_LOG=trace

BACKENDS=("wtx/crypto-alr wtx/_hack" "wtx/crypto-graviola" "wtx/crypto-ring" "wtx/crypto-ruco")
IS_CONCURRENT=("0" "1")

if [ ! -d "./boringssl" ]; then
    git clone --depth 1 --branch 0.20260713.0 https://github.com/google/boringssl
    rm -rf boringssl/.git
fi

if [ ! -f "./boringssl-config.json" ]; then
    cargo run --bin boringssl-config --features boringssl-config
fi

for is_concurrent in "${IS_CONCURRENT[@]}"; do
    export WTX_BORINGSSL_IS_CONCURRENT="$is_concurrent";

    for backend in "${BACKENDS[@]}"; do
        echo -e "\e[0;33m***** Testing with 'WTX_BORINGSSL_IS_CONCURRENT=$is_concurrent' and '$backend' *****\e[0m"

        cargo build --bin boringssl --features "boringssl $backend" --package wtx-internal
        pushd boringssl/ssl/test/runner
        go test -c
        ./runner.test \
            -allow-unimplemented \
            -num-workers 1 \
            -pipe \
            -shim-config ../../../../boringssl-config.json \
            -shim-path ../../../../target/debug/boringssl \
            -test.v
        popd
    done;
done;