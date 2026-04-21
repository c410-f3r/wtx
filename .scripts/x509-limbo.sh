#!/usr/bin/env bash

ARG=${1:-""}

if [ ! -e "limbo.json" ]; then
    curl -L -o limbo.json https://raw.githubusercontent.com/C2SP/x509-limbo/refs/heads/main/limbo.json
fi

cargo run --bin x509-limbo --features x509-limbo,crypto-aws-lc-rs,_hack --profile release
mv ./target/release/x509-limbo /tmp/aws-lc-rs

cargo run --bin x509-limbo --features x509-limbo,crypto-graviola --profile release
mv ./target/release/x509-limbo /tmp/graviola

cargo run --bin x509-limbo --features x509-limbo,crypto-ring --profile release
mv ./target/release/x509-limbo /tmp/ring

cargo run --bin x509-limbo --features x509-limbo,crypto-rust-crypto --profile release
mv ./target/release/x509-limbo /tmp/rust-crypto

if [ "$ARG" == "bench" ]; then
    hyperfine /tmp/aws-lc-rs /tmp/graviola /tmp/ring /tmp/rust-crypto
fi;
