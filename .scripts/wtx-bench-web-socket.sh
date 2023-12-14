#!/usr/bin/env bash

set -euxo pipefail

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

export CARGO_PROFILE_RELEASE_LTO=true

FLAGS="-Ccodegen-units=1 -Copt-level=3 -Cpanic=abort -Cstrip=symbols -Ctarget-cpu=native"

pushd /tmp
git clone https://github.com/c410-f3r/tokio-tungstenite || true
cd tokio-tungstenite
git checkout -t origin/bench || true
RUSTFLAGS="$FLAGS" cargo build --example echo-server --release
RUSTFLAGS="$FLAGS" cargo run --example echo-server --release 127.0.0.1:8081 &

cd /tmp
git clone --recursive https://github.com/c410-f3r/uWebSockets.git || true
cd uWebSockets
git checkout -t origin/bench || true
if [ ! -e ./EchoServer ]
then
    make examples
fi
./EchoServer 8082 &
popd

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-async-std --features async-std/attributes,simdutf8,web-socket-handshake --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-async-std --features async-std/attributes,simdutf8,web-socket-handshake --release 127.0.0.1:8083 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-glommio --features glommio,simdutf8,web-socket-handshake --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-glommio --features glommio,simdutf8,web-socket-handshake --release 127.0.0.1:8084 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-smol --features simdutf8,smol,web-socket-handshake --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-smol --features simdutf8,smol,web-socket-handshake --release 127.0.0.1:8085 &

RUSTFLAGS="$FLAGS" cargo build --example web-socket-server-echo-raw-tokio --features simdutf8,tokio,web-socket-handshake --release
RUSTFLAGS="$FLAGS" cargo run --example web-socket-server-echo-raw-tokio --features simdutf8,tokio,web-socket-handshake --release 127.0.0.1:8086 &

sleep 1

RUSTFLAGS="$FLAGS" cargo run --bin wtx-bench --release -- \
    web-socket \
    http://127.0.0.1:8081/tokio-tungstenite \
    http://127.0.0.1:8082/uWebSockets \
    http://127.0.0.1:8083/wtx-raw-async-std \
    http://127.0.0.1:8084/wtx-raw-glommio \
    http://127.0.0.1:8085/wtx-raw-smol \
    http://127.0.0.1:8086/wtx-raw-tokio
