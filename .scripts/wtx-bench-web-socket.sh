#!/usr/bin/env bash

set -euxo pipefail

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

export CARGO_PROFILE_RELEASE_LTO=true

FLAGS="-Ccodegen-units=1 -Copt-level=3 -Cpanic=abort -Cstrip=symbols -Ctarget-cpu=native"

pushd /tmp
git clone https://github.com/c410-f3r/tokio-tungstenite || true
cd tokio-tungstenite
RUSTFLAGS="$FLAGS" cargo build --example echo-server --release
RUSTFLAGS="$FLAGS" cargo run --example echo-server --release 127.0.0.1:8081 &
popd

WTX_FEATURES="atoi,memchr,simdutf8,web-socket-handshake"

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-async-std --features "async-std,$WTX_FEATURES" --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-async-std --features "async-std,$WTX_FEATURES" --release 127.0.0.1:8082 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-glommio --features "glommio,$WTX_FEATURES" --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-glommio --features "glommio,$WTX_FEATURES" --release 127.0.0.1:8083 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-smol --features "smol,$WTX_FEATURES" --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-smol --features "smol,$WTX_FEATURES" --release 127.0.0.1:8084 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-tokio --features "tokio,$WTX_FEATURES" --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-tokio --features "tokio,$WTX_FEATURES" --release 127.0.0.1:8085 &

sleep 1

RUSTFLAGS="$FLAGS" cargo run --bin wtx-bench --manifest-path wtx-bench/Cargo.toml --release -- \
    web-socket \
    http://127.0.0.1:8081/tokio-tungstenite \
    http://127.0.0.1:8082/wtx-raw-async-std \
    http://127.0.0.1:8083/wtx-raw-glommio \
    http://127.0.0.1:8084/wtx-raw-smol \
    http://127.0.0.1:8085/wtx-raw-tokio
