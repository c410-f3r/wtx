#!/usr/bin/env bash

set -euxo pipefail

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

pushd /tmp
git clone https://github.com/c410-f3r/fastwebsockets || true
cd fastwebsockets
git checkout -t origin/bench || true
RUSTFLAGS='-C target-cpu=native' cargo build --example echo_server --features simd,upgrade --release
RUSTFLAGS='-C target-cpu=native' cargo run --example echo_server --features simd,upgrade --release 127.0.0.1:8080 &

cd /tmp
git clone https://github.com/c410-f3r/websocket || true
cd websocket/examples/echo
git checkout -t origin/bench || true
go run server.go 127.0.0.1:8081 &

cd /tmp
git clone https://github.com/c410-f3r/tokio-tungstenite || true
cd tokio-tungstenite
git checkout -t origin/bench || true
RUSTFLAGS='-C target-cpu=native' cargo build --example echo-server --release
RUSTFLAGS='-C target-cpu=native' cargo run --example echo-server --release 127.0.0.1:8082 &

cd /tmp
git clone --recursive https://github.com/c410-f3r/uWebSockets.git || true
cd uWebSockets
git checkout -t origin/bench || true
if [ ! -e ./EchoServer ]
then
    make examples
fi
./EchoServer 8083 &
popd

RUSTFLAGS='-C target-cpu=native' cargo build --example web_socket_server_echo_hyper --features simdutf8,web-socket-hyper --release
RUSTFLAGS='-C target-cpu=native' cargo run --example web_socket_server_echo_hyper --features simdutf8,web-socket-hyper --release 127.0.0.1:8084 &

RUSTFLAGS='-C target-cpu=native' cargo build --example web_socket_server_echo_raw_async_std --features async-std,simdutf8,web-socket-handshake --release
RUSTFLAGS='-C target-cpu=native' cargo run --example web_socket_server_echo_raw_async_std --features async-std,simdutf8,web-socket-handshake --release 127.0.0.1:8085 &

RUSTFLAGS='-C target-cpu=native' cargo build --example web_socket_server_echo_raw_glommio --features glommio,simdutf8,web-socket-handshake --release
RUSTFLAGS='-C target-cpu=native' cargo run --example web_socket_server_echo_raw_glommio --features glommio,simdutf8,web-socket-handshake --release 127.0.0.1:8086 &

RUSTFLAGS='-C target-cpu=native' cargo build --example web_socket_server_echo_raw_tokio --features simdutf8,tokio,web-socket-handshake --release
RUSTFLAGS='-C target-cpu=native' cargo run --example web_socket_server_echo_raw_tokio --features simdutf8,tokio,web-socket-handshake --release 127.0.0.1:8087 &

sleep 1

RUSTFLAGS='-C target-cpu=native' cargo run --bin wtx-bench --release -- \
    http://127.0.0.1:8080/fastwebsockets \
    http://127.0.0.1:8081/gorilla-websocket \
    http://127.0.0.1:8082/tokio-tungstenite \
    http://127.0.0.1:8083/uWebSockets \
    http://127.0.0.1:8084/wtx-hyper \
    http://127.0.0.1:8085/wtx-raw-async-std \
    http://127.0.0.1:8086/wtx-raw-glommio \
    http://127.0.0.1:8087/wtx-raw-tokio
