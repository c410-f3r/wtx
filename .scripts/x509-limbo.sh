#!/usr/bin/env bash

if [ ! -e "x509-limbo" ]; then
    curl -L -o limbo.json https://raw.githubusercontent.com/C2SP/x509-limbo/refs/heads/main/limbo.json
fi

cargo run --bin x509-limbo --features x509-limbo < limbo.json