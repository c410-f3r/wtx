#!/usr/bin/env bash

set -euxo pipefail

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

export CARGO_PROFILE_RELEASE_LTO=true

FLAGS="-Ccodegen-units=1 -Copt-level=3 -Cpanic=abort -Cstrip=symbols -Ctarget-cpu=native"

pushd /tmp
git clone https://github.com/c410-f3r/tokio-tungstenite || true
cd tokio-tungstenite
git checkout -t origin/bench || true
RUSTFLAGS="$FLAGS" cargo build --example echo-server --profile bench
RUSTFLAGS="$FLAGS" cargo run --example echo-server --profile bench 127.0.0.1:8081 &
popd

FEATURES="atoi,memchr,simdutf8,web-socket-handshake"

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-async-std --features "async-std,$FEATURES" --profile bench
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-async-std --features "async-std,$FEATURES" --profile bench 127.0.0.1:8082 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-glommio --features "glommio,$FEATURES" --profile bench
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-glommio --features "glommio,$FEATURES" --profile bench 127.0.0.1:8083 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-smol --features "smol,$FEATURES" --profile bench
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-smol --features "smol,$FEATURES" --profile bench 127.0.0.1:8084 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-tokio --features "tokio,$FEATURES" --profile bench
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-tokio --features "tokio,$FEATURES" --profile bench 127.0.0.1:8085 &

sleep 1

RUSTFLAGS="$FLAGS" cargo run --bin wtx-bench --profile bench -- \
    web-socket \
    http://127.0.0.1:8081/tokio-tungstenite \
    http://127.0.0.1:8082/wtx-raw-async-std \
    http://127.0.0.1:8083/wtx-raw-glommio \
    http://127.0.0.1:8084/wtx-raw-smol \
    http://127.0.0.1:8085/wtx-raw-tokio
