#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --bin h2load --features h2load --profile deploy
cargo run --bin h2load --features h2load --profile deploy &
sleep 1

> /tmp/h2load.txt
h2load -c100 --log-file=/tmp/h2load.txt -m10 -n100000 --no-tls-proto=h2c http://localhost:9000
