#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
EXIT_CODE=0

if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --bin testssl --features testssl
cargo run --bin testssl --features testssl &> /tmp/testssl.txt &

mkdir -p /tmp/testssl
chmod 777 /tmp/testssl
rm -f /tmp/testssl/testssl.html

podman run \
	--network host \
	--rm \
	--userns=keep-id \
	-v /tmp/testssl:/tmp:Z \
	-v .certs:/tmp/certs:Z \
	ghcr.io/testssl/testssl.sh --add-ca /tmp/certs/root-ca.crt --debug 6 --full --htmlfile /tmp/testssl.html --openssl=/usr/bin/openssl --quiet --severity HIGH --wide localhost:9000 || EXIT_CODE=$?

if [ "$EXIT_CODE" -lt 10 ]; then
    exit 0
else
    exit "$EXIT_CODE"
fi