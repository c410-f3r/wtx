#!/usr/bin/env bash

set -euxo pipefail

BACKENDS=("wtx/crypto-aws-lc-rs wtx/_hack" "wtx/crypto-graviola" "wtx/crypto-ring" "wtx/crypto-ruco")

if [ ! -e "limbo.json" ]; then
    curl -L -o limbo.json https://raw.githubusercontent.com/C2SP/x509-limbo/refs/heads/main/limbo.json
fi

for backend in "${BACKENDS[@]}"; do
    echo -e "\e[0;33m***** Testing with '$backend' *****\e[0m"
    cargo run --bin x509-limbo --features "$backend x509-limbo" -p wtx-internal --release
done;