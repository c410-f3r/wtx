#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --bin h2load --features="wtx/http2,wtx/nightly" --profile deploy
cargo run --bin h2load --features="wtx/http2,wtx/nightly" --profile deploy &
sleep 1

h2load -c100 -m10 -n100000 --no-tls-proto=h2c http://localhost:9000
