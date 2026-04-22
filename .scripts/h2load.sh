#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

RUSTFLAGS='-C target-cpu=native' cargo build --bin h2load --features h2load --profile deploy
RUSTFLAGS='-C target-cpu=native' cargo run --bin h2load --features h2load --profile deploy &
sleep 1

# -c = Concurrent clients
# -m = Max concurrent streams
# -n = Requests across all clients
# -t = System threads
> /tmp/h2load.txt
h2load -c128 --log-file=/tmp/h2load.txt -m8 -n131072 --no-tls-proto=h2c -t4 http://localhost:9000
