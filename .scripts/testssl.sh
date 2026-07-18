#!/usr/bin/env bash

set -euxo pipefail

ARG=${1:-""}
BACKENDS=("wtx/crypto-aws-lc-rs wtx/_hack" "wtx/crypto-graviola" "wtx/crypto-ring" "wtx/crypto-ruco")
EXIT_CODE=0

if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

for backend in "${BACKENDS[@]}"; do
    echo -e "\e[0;33m***** Testing with '$backend' *****\e[0m"

	cargo build --bin testssl --features "$backend testssl" -p wtx-internal --release
	cargo run --bin testssl --features "$backend testssl" -p wtx-internal --release &> /tmp/testssl.txt &

	podman unshare rm -rf /tmp/testssl
	mkdir -p /tmp/testssl
	chmod 777 /tmp/testssl

	podman run \
		--network host \
		--rm \
		--userns=keep-id \
		-v /tmp/testssl:/tmp:Z \
		-v .certs:/tmp/certs:Z \
		ghcr.io/testssl/testssl.sh --add-ca /tmp/certs/root-ca.crt --debug 6 --full --htmlfile /tmp/testssl.html --openssl=/usr/bin/openssl --quiet --severity HIGH --wide localhost:9000 || EXIT_CODE=$?

	if [ "$EXIT_CODE" -gt 10 ]; then
		exit "$EXIT_CODE"
	fi
done;