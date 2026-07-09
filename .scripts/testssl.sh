#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --example tls-server --features tls-server
cargo run --example tls-server --features tls-server &> /tmp/testssl.txt &

mkdir -p /tmp/testssl
rm -f /tmp/testssl/testssl.html
podman run \
	--network host \
	--rm \
	--userns=keep-id \
	-v \
	/tmp/testssl:/tmp:Z ghcr.io/testssl/testssl.sh --debug 6 --htmlfile /tmp/testssl.html --quiet localhost:9000