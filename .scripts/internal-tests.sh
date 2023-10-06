#!/usr/bin/env bash

set -euxo pipefail

cargo install rust-tools --git https://github.com/c410-f3r/regular-crates

rt='rust-tools --template you-rust'

RUSTFLAGS="$($rt rust-flags)"

export CARGO_TARGET_DIR="$($rt target-dir)"
export RUST_BACKTRACE=1
export RUST_LOG=debug

$rt rustfmt
$rt clippy

$rt test-generic wtx
$rt test-with-features wtx arbitrary
$rt test-with-features wtx async-std
$rt test-with-features wtx base64
$rt test-with-features wtx embassy-net,_hack
$rt test-with-features wtx flate2
$rt test-with-features wtx futures-lite
$rt test-with-features wtx glommio
$rt test-with-features wtx httparse
$rt test-with-features wtx monoio
$rt test-with-features wtx rand
$rt test-with-features wtx rustls-pemfile
$rt test-with-features wtx sha1
$rt test-with-features wtx simdutf8
$rt test-with-features wtx smol
$rt test-with-features wtx std
$rt test-with-features wtx tokio
$rt test-with-features wtx tokio-rustls
$rt test-with-features wtx tokio-uring
$rt test-with-features wtx tracing
$rt test-with-features wtx web-socket-handshake
$rt test-with-features wtx webpki-roots

cargo check --bin autobahn-client --features "flate2,tokio/macros,tokio/rt,web-socket-handshake"
cargo check --bin autobahn-server --features "flate2,tokio/macros,tokio/rt,web-socket-handshake"
cargo check --bin web-socket-profiling --features "tokio/macros,tokio/rt"

cargo check --example web-socket-client-cli-raw-tokio-rustls --features "rustls-pemfile,tokio/io-std,tokio-rustls/tls12,web-socket-handshake,webpki-roots"
cargo check --example web-socket-server-echo-raw-async-std --features "async-std/attributes,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-glommio --features "glommio,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-monoio --features "monoio/iouring,monoio/legacy,monoio/macros,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-smol --features "smol,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-tokio --features "tokio/macros,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-tokio-uring --features "tokio-uring,web-socket-handshake"
cargo check --example web-socket-server-echo-raw-tokio-rustls --features "rustls-pemfile,tokio-rustls,web-socket-handshake"