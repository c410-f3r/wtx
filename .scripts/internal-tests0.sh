#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy

cargo miri test -p wtx

# WTX

$rt check-generic wtx
$rt test-with-features wtx ahash
$rt test-with-features wtx arbitrary
$rt test-with-features wtx async-send
$rt test-with-features wtx async-std
$rt test-with-features wtx atoi
$rt test-with-features wtx base64
$rt test-with-features wtx borsh
$rt test-with-features wtx chrono
$rt test-with-features wtx cl-aux
$rt test-with-features wtx client-api-framework
$rt test-with-features wtx crypto-common
$rt test-with-features wtx database
$rt test-with-features wtx digest
$rt test-with-features wtx embassy-net,_hack
$rt test-with-features wtx embassy-sync
$rt test-with-features wtx embassy-time
$rt test-with-features wtx embedded-tls
$rt test-with-features wtx fastrand
$rt test-with-features wtx flate2
$rt test-with-features wtx futures-lite
$rt test-with-features wtx glommio
$rt test-with-features wtx hashbrown
$rt test-with-features wtx hmac
$rt test-with-features wtx http1
$rt test-with-features wtx http2
$rt test-with-features wtx httparse
$rt test-with-features wtx memchr
$rt test-with-features wtx miniserde
$rt test-with-features wtx parking_lot
$rt test-with-features wtx pool
$rt test-with-features wtx postgres
$rt test-with-features wtx proptest
$rt test-with-features wtx protobuf
$rt test-with-features wtx rand
$rt test-with-features wtx ring
$rt test-with-features wtx rkyv,_hack
$rt test-with-features wtx rust_decimal
$rt test-with-features wtx rustls-pemfile
$rt test-with-features wtx rustls-pki-types 
$rt test-with-features wtx schema-manager
$rt test-with-features wtx schema-manager-dev
$rt test-with-features wtx serde
$rt test-with-features wtx serde_json
$rt test-with-features wtx serde_yaml
$rt test-with-features wtx serde-xml-rs
$rt test-with-features wtx sha1
$rt test-with-features wtx sha2
$rt test-with-features wtx simd-json
$rt test-with-features wtx simdutf8
$rt test-with-features wtx smol
$rt test-with-features wtx smoltcp,_hack
$rt test-with-features wtx std
$rt test-with-features wtx test-strategy
$rt test-with-features wtx tokio
$rt test-with-features wtx tokio-rustls
$rt test-with-features wtx tracing
$rt test-with-features wtx web-socket
$rt test-with-features wtx web-socket-handshake
$rt test-with-features wtx webpki-roots
$rt test-with-features wtx x509-certificate

$rt check-with-features wtx _bench
$rt check-with-features wtx _hack
$rt check-with-features wtx _integration-tests
$rt check-with-features wtx _tokio-rustls-client
$rt check-with-features wtx _tracing-subscriber
$rt test-with-features wtx _proptest

# WTX Macros

$rt test-generic wtx-macros

# WTX UI

$rt check-generic wtx-ui
$rt test-with-features wtx-ui embed-migrations
$rt test-with-features wtx-ui schema-manager
$rt test-with-features wtx-ui schema-manager-dev
$rt test-with-features wtx-ui http-client
$rt test-with-features wtx-ui web-socket

cargo check --bin autobahn-client --features "flate2,optimization,tokio/rt-multi-thread,web-socket-handshake"
cargo check --bin autobahn-server --features "flate2,optimization,pool,tokio/rt-multi-thread,web-socket-handshake"
cargo check --bin h2spec-server --features "http2,tokio"

cargo check --example database-client-postgres-tokio-rustls --features "_tokio-rustls-client,postgres"
cargo check --example http2-client-tokio-rustls --features "_tokio-rustls-client,http2"
cargo check --example http2-server-tokio-rustls --features "_tokio-rustls-client,http2,pool"
cargo check --example web-socket-client-raw-tokio-rustls --features "_tokio-rustls-client,web-socket-handshake"
cargo check --example web-socket-server-raw-tokio-rustls --features "pool,rustls-pemfile,tokio-rustls,web-socket-handshake"